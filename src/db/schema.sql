CREATE TABLE "Puzzle" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "puzzleId" TEXT NOT NULL,
    "fen" TEXT NOT NULL,
    "moves" TEXT NOT NULL,
    "rating" INTEGER NOT NULL,
    "themes" TEXT NOT NULL
);

CREATE UNIQUE INDEX "Puzzle_puzzleId_key" ON "Puzzle"("puzzleId");
CREATE INDEX "Puzzle_rating_idx" ON "Puzzle"("rating");
