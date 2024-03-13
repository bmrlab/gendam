/*
  Warnings:

  - You are about to drop the column `asssetObjectId` on the `FileHandlerTask` table. All the data in the column will be lost.
  - You are about to drop the column `asssetObjectId` on the `MediaData` table. All the data in the column will be lost.
  - You are about to drop the column `asssetObjectId` on the `FilePath` table. All the data in the column will be lost.
  - Added the required column `assetObjectId` to the `FileHandlerTask` table without a default value. This is not possible if the table is not empty.
  - Added the required column `assetObjectId` to the `MediaData` table without a default value. This is not possible if the table is not empty.

*/
-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_FileHandlerTask" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "assetObjectId" INTEGER NOT NULL,
    "taskType" TEXT NOT NULL,
    "startsAt" DATETIME,
    "endsAt" DATETIME,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "FileHandlerTask_assetObjectId_fkey" FOREIGN KEY ("assetObjectId") REFERENCES "AssetObject" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
INSERT INTO "new_FileHandlerTask" ("createdAt", "endsAt", "id", "startsAt", "taskType", "updatedAt") SELECT "createdAt", "endsAt", "id", "startsAt", "taskType", "updatedAt" FROM "FileHandlerTask";
DROP TABLE "FileHandlerTask";
ALTER TABLE "new_FileHandlerTask" RENAME TO "FileHandlerTask";
CREATE UNIQUE INDEX "FileHandlerTask_assetObjectId_taskType_key" ON "FileHandlerTask"("assetObjectId", "taskType");
CREATE TABLE "new_MediaData" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "width" INTEGER,
    "height" INTEGER,
    "duration" INTEGER,
    "bitRate" INTEGER,
    "description" TEXT,
    "assetObjectId" INTEGER NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "MediaData_assetObjectId_fkey" FOREIGN KEY ("assetObjectId") REFERENCES "AssetObject" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
INSERT INTO "new_MediaData" ("bitRate", "createdAt", "description", "duration", "height", "id", "updatedAt", "width") SELECT "bitRate", "createdAt", "description", "duration", "height", "id", "updatedAt", "width" FROM "MediaData";
DROP TABLE "MediaData";
ALTER TABLE "new_MediaData" RENAME TO "MediaData";
CREATE UNIQUE INDEX "MediaData_assetObjectId_key" ON "MediaData"("assetObjectId");
CREATE TABLE "new_FilePath" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "isDir" BOOLEAN NOT NULL,
    "materializedPath" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "assetObjectId" INTEGER,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "FilePath_assetObjectId_fkey" FOREIGN KEY ("assetObjectId") REFERENCES "AssetObject" ("id") ON DELETE SET NULL ON UPDATE CASCADE
);
INSERT INTO "new_FilePath" ("createdAt", "id", "isDir", "materializedPath", "name", "updatedAt") SELECT "createdAt", "id", "isDir", "materializedPath", "name", "updatedAt" FROM "FilePath";
DROP TABLE "FilePath";
ALTER TABLE "new_FilePath" RENAME TO "FilePath";
CREATE INDEX "FilePath_materializedPath_idx" ON "FilePath"("materializedPath");
CREATE UNIQUE INDEX "FilePath_materializedPath_name_key" ON "FilePath"("materializedPath", "name");
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
