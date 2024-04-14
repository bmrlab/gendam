/*
  Warnings:

  - You are about to drop the column `note` on the `AssetObject` table. All the data in the column will be lost.
  - You are about to drop the column `description` on the `MediaData` table. All the data in the column will be lost.
  - You are about to drop the column `mimeType` on the `MediaData` table. All the data in the column will be lost.
  - You are about to drop the column `size` on the `MediaData` table. All the data in the column will be lost.
  - Added the required column `size` to the `AssetObject` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "FilePath" ADD COLUMN "description" TEXT;

-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_AssetObject" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "hash" TEXT NOT NULL,
    "size" INTEGER NOT NULL,
    "mimeType" TEXT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL
);
INSERT INTO "new_AssetObject" ("createdAt", "hash", "id", "updatedAt") SELECT "createdAt", "hash", "id", "updatedAt" FROM "AssetObject";
DROP TABLE "AssetObject";
ALTER TABLE "new_AssetObject" RENAME TO "AssetObject";
CREATE UNIQUE INDEX "AssetObject_hash_key" ON "AssetObject"("hash");
CREATE TABLE "new_MediaData" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "width" INTEGER,
    "height" INTEGER,
    "duration" INTEGER,
    "bitRate" INTEGER,
    "hasAudio" BOOLEAN,
    "assetObjectId" INTEGER NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "MediaData_assetObjectId_fkey" FOREIGN KEY ("assetObjectId") REFERENCES "AssetObject" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
INSERT INTO "new_MediaData" ("assetObjectId", "bitRate", "createdAt", "duration", "hasAudio", "height", "id", "updatedAt", "width") SELECT "assetObjectId", "bitRate", "createdAt", "duration", "hasAudio", "height", "id", "updatedAt", "width" FROM "MediaData";
DROP TABLE "MediaData";
ALTER TABLE "new_MediaData" RENAME TO "MediaData";
CREATE UNIQUE INDEX "MediaData_assetObjectId_key" ON "MediaData"("assetObjectId");
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
