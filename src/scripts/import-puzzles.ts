import { createReadStream } from "node:fs";
import { resolve } from "node:path";
import readline from "node:readline";
import { fileURLToPath } from "node:url";
import { prisma } from "../init/prisma.js";

const CSV_PATH = resolve(
	process.cwd(),
	process.env.PUZZLE_DB_PATH ?? "lichess_db_puzzle.csv",
);
const IMPORT_MAX_ROWS = Number.parseInt(process.env.IMPORT_MAX_ROWS ?? "", 10);
const INSERT_BATCH_SIZE = 1000;

type ParsedPuzzle = {
	puzzleId: string;
	fen: string;
	moves: string;
	rating: number;
	themes: string;
};

function parseCsvLine(line: string): string[] {
	const values: string[] = [];
	let current = "";
	let inQuotes = false;

	for (let i = 0; i < line.length; i += 1) {
		const ch = line[i];

		if (ch === '"') {
			if (inQuotes && line[i + 1] === '"') {
				current += '"';
				i += 1;
			} else {
				inQuotes = !inQuotes;
			}
			continue;
		}

		if (ch === "," && !inQuotes) {
			values.push(current);
			current = "";
			continue;
		}

		current += ch;
	}

	values.push(current);
	return values;
}

function toInt(raw: string, fieldName: string, lineNumber: number): number {
	const parsed = Number.parseInt(raw, 10);
	if (Number.isNaN(parsed)) {
		throw new Error(
			`Invalid integer for ${fieldName} at line ${lineNumber}: ${raw}`,
		);
	}
	return parsed;
}

function toPuzzle(columns: string[], lineNumber: number): ParsedPuzzle {
	if (columns.length < 8) {
		throw new Error(
			`Expected at least 8 columns at line ${lineNumber}, got ${columns.length}`,
		);
	}

	return {
		puzzleId: columns[0],
		fen: columns[1],
		moves: columns[2],
		rating: toInt(columns[3], "rating", lineNumber),
		themes: columns[7],
	};
}

async function flushBatch(batch: ParsedPuzzle[]): Promise<void> {
	if (batch.length === 0) {
		return;
	}

	await prisma.puzzle.createMany({
		data: batch,
	});
}

export async function importPuzzles(): Promise<void> {
	let totalLines = 0;
	let imported = 0;
	let skipped = 0;
	let lastProgressLog = 0;
	let batch: ParsedPuzzle[] = [];

	const stream = createReadStream(CSV_PATH, { encoding: "utf8" });
	const rl = readline.createInterface({ input: stream, crlfDelay: Infinity });

	try {
		console.log("Clearing existing puzzles...");
		await prisma.puzzle.deleteMany();
		await prisma.$queryRaw`DELETE FROM sqlite_sequence WHERE name = 'Puzzle';`;

		for await (const line of rl) {
			totalLines += 1;

			// Skip header row.
			if (totalLines === 1) {
				continue;
			}

			if (line.trim() === "") {
				skipped += 1;
				continue;
			}

			const columns = parseCsvLine(line);
			const puzzle = toPuzzle(columns, totalLines);
			batch.push(puzzle);

			if (batch.length >= INSERT_BATCH_SIZE) {
				await flushBatch(batch);
				imported += batch.length;
				batch = [];

				if (imported - lastProgressLog >= 10_000) {
					lastProgressLog = imported;
					console.log(`Imported ${imported} puzzles...`);
				}
			}

			if (
				!Number.isNaN(IMPORT_MAX_ROWS) &&
				IMPORT_MAX_ROWS > 0 &&
				imported + batch.length >= IMPORT_MAX_ROWS
			) {
				console.log(
					`Reached IMPORT_MAX_ROWS=${IMPORT_MAX_ROWS}. Stopping early.`,
				);
				break;
			}
		}

		if (batch.length > 0) {
			if (
				!Number.isNaN(IMPORT_MAX_ROWS) &&
				IMPORT_MAX_ROWS > 0 &&
				imported + batch.length > IMPORT_MAX_ROWS
			) {
				batch = batch.slice(0, IMPORT_MAX_ROWS - imported);
			}

			await flushBatch(batch);
			imported += batch.length;
		}

		console.log(
			`Import complete. Imported ${imported} puzzles, skipped ${skipped} rows.`,
		);
	} finally {
		rl.close();
		stream.close();
		await prisma.$disconnect();
	}
}

function isDirectExecution(): boolean {
	if (!process.argv[1]) {
		return false;
	}

	return resolve(process.argv[1]) === fileURLToPath(import.meta.url);
}

if (isDirectExecution()) {
	importPuzzles().catch((error: unknown) => {
		console.error("Puzzle import failed", error);
		process.exitCode = 1;
	});
}
