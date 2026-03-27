-- CreateTable
CREATE TABLE "Puzzle" (
    "puzzleId" TEXT NOT NULL PRIMARY KEY,
    "fen" TEXT NOT NULL,
    "moves" TEXT NOT NULL,
    "rating" INTEGER NOT NULL,
    "ratingDeviation" INTEGER NOT NULL,
    "popularity" INTEGER NOT NULL,
    "nbPlays" INTEGER NOT NULL,
    "themes" TEXT NOT NULL,
    "gameUrl" TEXT NOT NULL,
    "openingTags" TEXT
);

-- CreateIndex
CREATE INDEX "Puzzle_rating_idx" ON "Puzzle"("rating");
