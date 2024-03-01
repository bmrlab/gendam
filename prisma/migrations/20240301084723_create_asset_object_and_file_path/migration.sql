-- CreateTable
CREATE TABLE "FilePath" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "isDir" BOOLEAN NOT NULL,
    "materializedPath" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "asssetObjectId" INTEGER,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "FilePath_asssetObjectId_fkey" FOREIGN KEY ("asssetObjectId") REFERENCES "AssetObject" ("id") ON DELETE SET NULL ON UPDATE CASCADE
);

-- CreateTable
CREATE TABLE "AssetObject" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "note" TEXT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL
);

-- CreateTable
CREATE TABLE "VideoClip" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "fileIdentifier" TEXT NOT NULL,
    "startTimestamp" INTEGER NOT NULL,
    "endTimestamp" INTEGER NOT NULL,
    "caption" TEXT
);

-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_VideoFrame" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "fileIdentifier" TEXT NOT NULL,
    "timestamp" INTEGER NOT NULL,
    "videoClipId" INTEGER,
    CONSTRAINT "VideoFrame_videoClipId_fkey" FOREIGN KEY ("videoClipId") REFERENCES "VideoClip" ("id") ON DELETE SET NULL ON UPDATE CASCADE
);
INSERT INTO "new_VideoFrame" ("createdAt", "fileIdentifier", "id", "timestamp", "updatedAt") SELECT "createdAt", "fileIdentifier", "id", "timestamp", "updatedAt" FROM "VideoFrame";
DROP TABLE "VideoFrame";
ALTER TABLE "new_VideoFrame" RENAME TO "VideoFrame";
CREATE UNIQUE INDEX "VideoFrame_fileIdentifier_timestamp_key" ON "VideoFrame"("fileIdentifier", "timestamp");
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;

-- CreateIndex
CREATE INDEX "FilePath_materializedPath_idx" ON "FilePath"("materializedPath");

-- CreateIndex
CREATE UNIQUE INDEX "FilePath_materializedPath_name_key" ON "FilePath"("materializedPath", "name");
