use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use sqlx::SqlitePool;
use tokio::fs::File;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::stores;
use crate::stores::puzzles::NewPuzzle;

const BATCH_SIZE: usize = 1_000;

pub async fn puzzles(pool: &SqlitePool, only_if_empty: bool) -> Result<()> {
    let mut transaction = pool.begin_with("BEGIN IMMEDIATE").await?;
    if only_if_empty {
        if stores::puzzles::has_puzzles(&mut *transaction).await? {
            return Ok(());
        }
        println!("no puzzles found in database");
    }

    let path = std::env::var_os("PUZZLE_DB_PATH")
        .map_or_else(|| PathBuf::from("lichess_db_puzzle.csv"), PathBuf::from);
    let row_limit = std::env::var("IMPORT_MAX_ROWS")
        .ok()
        .and_then(|value| integer_prefix(&value).parse::<usize>().ok())
        .filter(|limit| *limit > 0);

    let file = File::open(&path)
        .await
        .with_context(|| format!("could not open puzzle CSV {}", path.display()))?;
    let mut lines = BufReader::new(file).lines();

    println!("Clearing existing puzzles...");
    stores::puzzles::clear(&mut *transaction).await?;

    let mut line_number = 1;
    let mut imported = 0;
    let mut skipped = 0;
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    // The first physical line is the header.
    lines.next_line().await?;
    while let Some(line) = lines.next_line().await? {
        line_number += 1;
        if line.trim_matches(csv_whitespace).is_empty() {
            skipped += 1;
            continue;
        }

        batch.push(parse_puzzle(&line, line_number)?);
        if batch.len() == BATCH_SIZE {
            stores::puzzles::insert_batch(&mut *transaction, &batch).await?;
            imported += batch.len();
            batch.clear();

            if imported % 10_000 == 0 {
                println!("Imported {imported} puzzles...");
            }
        }

        if let Some(limit) = row_limit
            && imported + batch.len() == limit
        {
            println!("Reached IMPORT_MAX_ROWS={limit}. Stopping early.");
            break;
        }
    }

    if !batch.is_empty() {
        stores::puzzles::insert_batch(&mut *transaction, &batch).await?;
        imported += batch.len();
    }
    transaction.commit().await?;
    println!("Import complete. Imported {imported} puzzles, skipped {skipped} rows.");
    Ok(())
}

fn parse_puzzle(line: &str, line_number: usize) -> Result<NewPuzzle> {
    let mut columns = Vec::new();
    let mut field = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                field.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' if !quoted => columns.push(std::mem::take(&mut field)),
            _ => field.push(ch),
        }
    }
    columns.push(field);

    if columns.len() < 8 {
        bail!(
            "Expected at least 8 columns at line {line_number}, got {}",
            columns.len()
        );
    }

    let rating = integer_prefix(&columns[3]).parse().with_context(|| {
        format!(
            "Invalid integer for rating at line {line_number}: {}",
            columns[3]
        )
    })?;
    Ok(NewPuzzle {
        puzzle_id: std::mem::take(&mut columns[0]),
        fen: std::mem::take(&mut columns[1]),
        moves: std::mem::take(&mut columns[2]),
        rating,
        themes: std::mem::take(&mut columns[7]),
    })
}

fn integer_prefix(value: &str) -> &str {
    let value = value.trim_start_matches(csv_whitespace);
    let sign_len = usize::from(value.starts_with(['+', '-']));
    let digits = value[sign_len..]
        .bytes()
        .take_while(u8::is_ascii_digit)
        .count();
    &value[..sign_len + digits]
}

fn csv_whitespace(ch: char) -> bool {
    (ch.is_whitespace() && ch != '\u{85}') || ch == '\u{feff}'
}
