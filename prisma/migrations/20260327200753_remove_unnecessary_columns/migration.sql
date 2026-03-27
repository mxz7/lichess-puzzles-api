/*
  Warnings:

  - You are about to drop the column `gameUrl` on the `Puzzle` table. All the data in the column will be lost.
  - You are about to drop the column `nbPlays` on the `Puzzle` table. All the data in the column will be lost.
  - You are about to drop the column `openingTags` on the `Puzzle` table. All the data in the column will be lost.
  - You are about to drop the column `popularity` on the `Puzzle` table. All the data in the column will be lost.
  - You are about to drop the column `ratingDeviation` on the `Puzzle` table. All the data in the column will be lost.

*/
-- RedefineTables
PRAGMA defer_foreign_keys=ON;
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_Puzzle" (
    "puzzleId" TEXT NOT NULL PRIMARY KEY,
    "fen" TEXT NOT NULL,
    "moves" TEXT NOT NULL,
    "rating" INTEGER NOT NULL,
    "themes" TEXT NOT NULL
);
INSERT INTO "new_Puzzle" ("fen", "moves", "puzzleId", "rating", "themes") SELECT "fen", "moves", "puzzleId", "rating", "themes" FROM "Puzzle";
DROP TABLE "Puzzle";
ALTER TABLE "new_Puzzle" RENAME TO "Puzzle";
CREATE INDEX "Puzzle_rating_idx" ON "Puzzle"("rating");
PRAGMA foreign_keys=ON;
PRAGMA defer_foreign_keys=OFF;
