/*
  Warnings:

  - The primary key for the `Puzzle` table will be changed. If it partially fails, the table could be left without primary key constraint.
  - Added the required column `id` to the `Puzzle` table without a default value. This is not possible if the table is not empty.

*/
-- RedefineTables
PRAGMA defer_foreign_keys=ON;
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_Puzzle" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "puzzleId" TEXT NOT NULL,
    "fen" TEXT NOT NULL,
    "moves" TEXT NOT NULL,
    "rating" INTEGER NOT NULL,
    "themes" TEXT NOT NULL
);
INSERT INTO "new_Puzzle" ("fen", "moves", "puzzleId", "rating", "themes") SELECT "fen", "moves", "puzzleId", "rating", "themes" FROM "Puzzle";
DROP TABLE "Puzzle";
ALTER TABLE "new_Puzzle" RENAME TO "Puzzle";
CREATE UNIQUE INDEX "Puzzle_puzzleId_key" ON "Puzzle"("puzzleId");
CREATE INDEX "Puzzle_rating_idx" ON "Puzzle"("rating");
PRAGMA foreign_keys=ON;
PRAGMA defer_foreign_keys=OFF;
