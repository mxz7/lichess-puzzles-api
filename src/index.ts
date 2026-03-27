import { serve } from "@hono/node-server";
import "dotenv/config";
import { Hono } from "hono";
import { pinoLogger } from "hono-pino";
import { checkPuzzles } from "./init/puzzles.js";
import { puzzleRouter } from "./routes/puzzle.js";

const app = new Hono();

app.use(
	pinoLogger({
		pino: {
			level: "debug",
			transport: {
				target: "pino/file",
				options: { destination: process.env.LOG_OUTPUT },
			},
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
