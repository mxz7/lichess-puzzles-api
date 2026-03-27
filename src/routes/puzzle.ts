import { Hono } from "hono";
import { prisma } from "../init/prisma.js";
import {
	difficultyRanges,
	getRandomPuzzle,
	getRandomPuzzleByDifficulty,
	type Difficulty,
} from "../services/puzzle.js";

const router = new Hono();

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
					"Invalid difficulty. Use one of: beginner, easy, medium, hard, expert, grandmaster",
				status: 400,
			},
			400,
		);
	}

	const difficulty = difficultyParam as Difficulty;
	const puzzle = await getRandomPuzzleByDifficulty(difficulty);

	if (!puzzle) {
		return c.json(
			{ error: `No puzzles found for difficulty '${difficulty}'`, status: 404 },
			404,
		);
	}

	return c.json(puzzle);
});

router.get("/random", async (c) => {
	const puzzle = await getRandomPuzzle();

	if (!puzzle) {
		return c.json({ error: "No puzzles found", status: 404 }, 404);
	}

	return c.json(puzzle);
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
