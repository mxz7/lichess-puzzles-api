use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use axum::extract::Request;
use axum::http::{StatusCode, header::COOKIE};
use grow_grafana_loki::{Entry, LokiClient, LokiConfig, PushOptions, Stream};
use serde::Serialize;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_stream::{StreamExt, wrappers::ReceiverStream};

const LOKI_BUFFER_SIZE: usize = 10_000;

pub struct Logger {
    hostname: String,
    next_id: AtomicU64,
    loki: Option<mpsc::Sender<RequestLog>>,
}

pub struct PendingRequest {
    id: u64,
    info: RequestInfo,
    started: Instant,
}

#[derive(Serialize)]
struct RequestInfo {
    url: String,
    method: String,
    headers: BTreeMap<String, String>,
}

#[derive(Serialize)]
struct ResponseInfo {
    status: u16,
    headers: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RequestLog {
    level: u8,
    time: u64,
    pid: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    hostname: Option<String>,
    req: RequestInfo,
    res: ResponseInfo,
    req_id: u64,
    response_time: u64,
    msg: String,
}

impl Logger {
    pub fn new() -> Result<(Self, Option<JoinHandle<()>>)> {
        let hostname = hostname::get()?.to_string_lossy().into_owned();
        let (loki, worker) = if let Some(client) = loki_client()? {
            println!("using loki transport");
            let (sender, receiver) = mpsc::channel(LOKI_BUFFER_SIZE);
            let worker = tokio::spawn(push_logs(client, hostname.clone(), receiver));
            (Some(sender), Some(worker))
        } else {
            println!("missing loki credentials, skipping loki transport");
            (None, None)
        };

        Ok((
            Self {
                hostname,
                next_id: AtomicU64::new(1),
                loki,
            },
            worker,
        ))
    }

    pub fn start(&self, request: &Request, path: &str) -> PendingRequest {
        let mut headers = BTreeMap::<String, String>::new();
        for (name, value) in request.headers() {
            let value: String = value.as_bytes().iter().copied().map(char::from).collect();
            let separator = if name == COOKIE { "; " } else { ", " };
            headers
                .entry(name.to_string())
                .and_modify(|existing| {
                    existing.push_str(separator);
                    existing.push_str(&value);
                })
                .or_insert(value);
        }

        PendingRequest {
            id: self.next_id.fetch_add(1, Ordering::Relaxed),
            info: RequestInfo {
                url: path.to_owned(),
                method: request.method().to_string(),
                headers,
            },
            started: Instant::now(),
        }
    }

    pub fn finish(&self, request: PendingRequest, status: StatusCode, error: Option<String>) {
        let log = RequestLog {
            level: if error.is_some() { 50 } else { 30 },
            time: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            pid: std::process::id(),
            hostname: self.loki.is_none().then(|| self.hostname.clone()),
            req: request.info,
            res: ResponseInfo {
                status: status.as_u16(),
                // Hono's Headers object serialized to an empty object.
                headers: BTreeMap::new(),
            },
            req_id: request.id,
            response_time: (request.started.elapsed().as_micros() as u64 + 500) / 1_000,
            msg: error.unwrap_or_else(|| "Request completed".to_owned()),
        };

        if let Some(sender) = &self.loki {
            let _ = sender.try_send(log);
        } else {
            println!(
                "{}",
                serde_json::to_string(&log).expect("serialize request log")
            );
        }
    }
}

fn loki_client() -> Result<Option<LokiClient>> {
    let get = |name| std::env::var(name).ok().filter(|value| !value.is_empty());
    let (Some(host), Some(username), Some(password)) =
        (get("LOKI_HOST"), get("LOKI_USERNAME"), get("LOKI_PASSWORD"))
    else {
        return Ok(None);
    };

    let config = LokiConfig::builder()
        .url(host)
        .username(username)
        .password(password)
        .tenant_id("max")
        .allowed_labels(["service_name", "hostname", "level"])
        .max_retries(0)
        .build()?;
    Ok(Some(LokiClient::new(config)?))
}

async fn push_logs(client: LokiClient, hostname: String, receiver: mpsc::Receiver<RequestLog>) {
    let batches =
        ReceiverStream::new(receiver).chunks_timeout(LOKI_BUFFER_SIZE, Duration::from_secs(5));
    tokio::pin!(batches);

    while let Some(batch) = batches.next().await {
        let streams: Vec<_> = batch
            .into_iter()
            .map(|log| {
                let level = if log.level == 50 { "error" } else { "info" };
                let timestamp = UNIX_EPOCH + Duration::from_millis(log.time);
                let line = serde_json::to_string(&log).expect("serialize request log");
                Stream::new([
                    ("service_name", "lichess-puzzles-api"),
                    ("hostname", hostname.as_str()),
                    ("level", level),
                ])
                .entry(Entry::new(line).at(timestamp))
            })
            .collect();
        if let Err(error) = client.push(&streams, PushOptions::default()).await {
            eprintln!("Loki push failed: {error}");
        }
    }
}
