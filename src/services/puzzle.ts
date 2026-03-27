import type { Puzzle } from "../generated/prisma/client.js";
import { Prisma } from "../generated/prisma/client.js";
import { prisma } from "../init/prisma.js";

export const difficultyRanges = {
	beginner: { min: 0, maxExclusive: 1000 },
	easy: { min: 1000, maxExclusive: 1300 },
	medium: { min: 1300, maxExclusive: 1600 },
	hard: { min: 1600, maxExclusive: 1900 },
	expert: { min: 1900, maxExclusive: 2499 },
	grandmaster: { min: 2500, maxExclusive: null },
} as const;

export type Difficulty = keyof typeof difficultyRanges;

const COUNT_CACHE_TTL_MS = 24 * 60 * 60 * 1000;
let cachedPuzzleCount: number | null = null;
let cachedPuzzleCountUpdatedAt = 0;

function randomIntInclusive(min: number, max: number): number {
	return Math.floor(Math.random() * (max - min + 1)) + min;
}

async function getPuzzleCountCached(forceRefresh = false): Promise<number> {
	const now = Date.now();
	const isStale = now - cachedPuzzleCountUpdatedAt >= COUNT_CACHE_TTL_MS;

	if (forceRefresh || cachedPuzzleCount === null || isStale) {
		cachedPuzzleCount = await prisma.puzzle.count();
		cachedPuzzleCountUpdatedAt = now;
	}

	return cachedPuzzleCount;
}

export async function getRandomPuzzle(): Promise<Puzzle | null> {
	const puzzleCount = await getPuzzleCountCached();

	if (puzzleCount <= 0) {
		return null;
	}

	const randomId = randomIntInclusive(1, puzzleCount);
	return prisma.puzzle.findUnique({
		where: { id: randomId },
	});
}

export async function getRandomPuzzleByDifficulty(
	difficulty: Difficulty,
): Promise<Puzzle | null> {
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

	return puzzles[0] ?? null;
}
