/*
  Warnings:

  - Made the column `hash` on table `AssetObject` required. This step will fail if there are existing NULL values in that column.

*/
-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_AssetObject" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "note" TEXT,
    "hash" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL
);
INSERT INTO "new_AssetObject" ("createdAt", "hash", "id", "note", "updatedAt") SELECT "createdAt", "hash", "id", "note", "updatedAt" FROM "AssetObject";
DROP TABLE "AssetObject";
ALTER TABLE "new_AssetObject" RENAME TO "AssetObject";
CREATE UNIQUE INDEX "AssetObject_hash_key" ON "AssetObject"("hash");
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
