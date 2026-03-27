import { Hono } from "hono";
import type { Puzzle } from "../generated/prisma/client.js";
import { Prisma } from "../generated/prisma/client.js";
import { prisma } from "../init/prisma.js";

const router = new Hono();

const difficultyRanges = {
	beginner: { min: 0, maxExclusive: 1200 },
	easy: { min: 1200, maxExclusive: 1500 },
	medium: { min: 1500, maxExclusive: 1800 },
	hard: { min: 1800, maxExclusive: 2100 },
	expert: { min: 2100, maxExclusive: null },
} as const;

type Difficulty = keyof typeof difficultyRanges;

router.get("/random/:difficulty", async (c) => {
	const rawDifficulty = c.req.param("difficulty");
	if (!rawDifficulty) {
		return c.json({ error: "Missing difficulty parameter", status: 400 }, 400);
	}

	const difficultyParam = rawDifficulty.toLowerCase();

	if (!(difficultyParam in difficultyRanges)) {
		return c.json(
			{
				error:
					"Invalid difficulty. Use one of: beginner, easy, medium, hard, expert",
				status: 400,
			},
			400,
		);
	}

	const difficulty = difficultyParam as Difficulty;
	const range = difficultyRanges[difficulty];

	const whereClause =
		range.maxExclusive === null
			? Prisma.sql`WHERE "rating" >= ${range.min}`
			: Prisma.sql`WHERE "rating" >= ${range.min} AND "rating" < ${range.maxExclusive}`;

	const puzzles: Puzzle[] = await prisma.$queryRaw`
		SELECT *
		FROM "Puzzle"
		${whereClause}
		ORDER BY RANDOM()
		LIMIT 1
	`;

	if (!puzzles || puzzles.length === 0) {
		return c.json(
			{ error: `No puzzles found for difficulty '${difficulty}'`, status: 404 },
			404,
		);
	}

	return c.json(puzzles[0]);
});

router.get("/random", async (c) => {
	const puzzles: Puzzle[] =
		await prisma.$queryRaw`SELECT * FROM "Puzzle" ORDER BY RANDOM() LIMIT 1`;

	if (!puzzles || puzzles.length === 0) {
		return c.json({ error: "No puzzles found", status: 404 }, 404);
	}

	return c.json(puzzles[0]);
});

router.get("/{id}", async (c) => {
	const puzzle = await prisma.puzzle.findUnique({
		where: { puzzleId: c.req.param("id") },
	});
	if (!puzzle) {
		return c.json({ error: "Puzzle not found", status: 404 }, 404);
	}

	return c.json(puzzle);
});

export { router as puzzleRouter };
