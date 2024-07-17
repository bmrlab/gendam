-- CreateTable
CREATE TABLE "Trash" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "isDir" BOOLEAN NOT NULL,
    "originParentId" INTEGER,
    "materializedPath" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "description" TEXT,
    "assetObjectId" INTEGER,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "Trash_assetObjectId_fkey" FOREIGN KEY ("assetObjectId") REFERENCES "AssetObject" ("id") ON DELETE SET NULL ON UPDATE CASCADE
);

-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_AssetObject" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "hash" TEXT NOT NULL,
    "size" INTEGER NOT NULL,
    "mimeType" TEXT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "status" INTEGER NOT NULL DEFAULT 0
);
INSERT INTO "new_AssetObject" ("createdAt", "hash", "id", "mimeType", "size", "updatedAt") SELECT "createdAt", "hash", "id", "mimeType", "size", "updatedAt" FROM "AssetObject";
DROP TABLE "AssetObject";
ALTER TABLE "new_AssetObject" RENAME TO "AssetObject";
CREATE UNIQUE INDEX "AssetObject_hash_key" ON "AssetObject"("hash");
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;

-- CreateIndex
CREATE INDEX "Trash_materializedPath_idx" ON "Trash"("materializedPath");

-- CreateIndex
CREATE UNIQUE INDEX "Trash_materializedPath_name_key" ON "Trash"("materializedPath", "name");
