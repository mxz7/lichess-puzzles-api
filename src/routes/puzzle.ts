import { Hono } from "hono";
import type { Puzzle } from "../generated/prisma/client.js";
import { prisma } from "../init/prisma.js";

const router = new Hono();

router.get("/{id}", async (c) => {
	const puzzle = await prisma.puzzle.findUnique({
		where: { puzzleId: c.req.param("id") },
	});
	if (!puzzle) {
		return c.json({ error: "Puzzle not found", status: 404 }, 404);
	}

	return c.json(puzzle);
});

router.get("/random", async (c) => {
	const puzzles: Puzzle[] =
		await prisma.$queryRaw`SELECT * FROM "Puzzle" ORDER BY RANDOM() LIMIT 1`;

	if (!puzzles || puzzles.length === 0) {
		return c.json({ error: "No puzzles found", status: 404 }, 404);
	}

	return c.json(puzzles[0]);
});

export { router as puzzleRouter };
