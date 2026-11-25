CREATE TABLE "new_Puzzle" (
    "puzzleId" TEXT NOT NULL PRIMARY KEY,
    "fen" TEXT NOT NULL,
    "moves" TEXT NOT NULL,
    "rating" INTEGER NOT NULL,
    "themes" TEXT NOT NULL
);

INSERT INTO "new_Puzzle" ("fen", "moves", "puzzleId", "rating", "themes")
SELECT "fen", "moves", "puzzleId", "rating", "themes" FROM "Puzzle";

DROP TABLE "Puzzle";
ALTER TABLE "new_Puzzle" RENAME TO "Puzzle";
CREATE INDEX "Puzzle_rating_idx" ON "Puzzle"("rating");
