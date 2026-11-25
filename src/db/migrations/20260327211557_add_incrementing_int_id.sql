CREATE TABLE "new_Puzzle" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "puzzleId" TEXT NOT NULL,
    "fen" TEXT NOT NULL,
    "moves" TEXT NOT NULL,
    "rating" INTEGER NOT NULL,
    "themes" TEXT NOT NULL
);

INSERT INTO "new_Puzzle" ("fen", "moves", "puzzleId", "rating", "themes")
SELECT "fen", "moves", "puzzleId", "rating", "themes" FROM "Puzzle";

DROP TABLE "Puzzle";
ALTER TABLE "new_Puzzle" RENAME TO "Puzzle";
CREATE UNIQUE INDEX "Puzzle_puzzleId_key" ON "Puzzle"("puzzleId");
CREATE INDEX "Puzzle_rating_idx" ON "Puzzle"("rating");
