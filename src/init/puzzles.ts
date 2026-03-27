import { importPuzzles } from "../scripts/import-puzzles.js";
import { prisma } from "./prisma.js";

export async function checkPuzzles() {
	const puzzle = await prisma.puzzle.findFirst();

	if (!puzzle) {
		console.log("no puzzles found in database");
		importPuzzles();
	}
}
