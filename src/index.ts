import { serve } from "@hono/node-server";
import "dotenv/config";
import { Hono } from "hono";
import { pinoLogger } from "hono-pino";
import { checkPuzzles } from "./init/puzzles.js";
import { puzzleRouter } from "./routes/puzzle.js";

const app = new Hono();

function buildTransport() {
	if (
		!process.env.LOKI_USERNAME ||
		!process.env.LOKI_PASSWORD ||
		!process.env.LOKI_HOST
	) {
		console.log("missing loki credentials, skipping loki transport");
		return undefined;
	}

	console.log("using loki transport");

	return {
		target: "pino-loki",
		options: {
			host: process.env.LOKI_HOST,
			basicAuth: {
				username: process.env.LOKI_USERNAME,
				password: process.env.LOKI_PASSWORD,
			},
			labels: { service_name: "lichess-puzzles-api" },
			headers: { "X-Scope-OrgID": "max" },
		},
	};
}

app.use(
	pinoLogger({
		pino: {
			level: "debug",
			transport: buildTransport(),
		},
	}),
);

app.get("/", (c) => {
	return c.json({ status: "ok", message: "powered by lichess!" });
});

app.route("/puzzles", puzzleRouter);

checkPuzzles();

serve(
	{
		fetch: app.fetch,
		port: 3000,
	},
	(info) => {
		console.log(`Server is running on http://localhost:${info.port}`);
	},
);
