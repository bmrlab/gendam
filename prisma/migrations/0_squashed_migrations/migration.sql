-- CreateTable
CREATE TABLE "FilePath" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "isDir" BOOLEAN NOT NULL,
    "materializedPath" TEXT NOT NULL,
    "name" TEXT NOT NULL,
    "description" TEXT,
    "assetObjectId" INTEGER,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "FilePath_assetObjectId_fkey" FOREIGN KEY ("assetObjectId") REFERENCES "AssetObject" ("id") ON DELETE SET NULL ON UPDATE CASCADE
);

-- CreateTable
CREATE TABLE "AssetObject" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "hash" TEXT NOT NULL,
    "size" INTEGER NOT NULL,
    "mimeType" TEXT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL
);

-- CreateTable
CREATE TABLE "MediaData" (
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

-- CreateTable
CREATE TABLE "FileHandlerTask" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "assetObjectId" INTEGER NOT NULL,
    "taskType" TEXT NOT NULL,
    "exitCode" INTEGER,
    "exitMessage" TEXT,
    "startsAt" DATETIME,
    "endsAt" DATETIME,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "FileHandlerTask_assetObjectId_fkey" FOREIGN KEY ("assetObjectId") REFERENCES "AssetObject" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateIndex
CREATE INDEX "FilePath_materializedPath_idx" ON "FilePath"("materializedPath");

-- CreateIndex
CREATE UNIQUE INDEX "FilePath_materializedPath_name_key" ON "FilePath"("materializedPath", "name");

-- CreateIndex
CREATE UNIQUE INDEX "AssetObject_hash_key" ON "AssetObject"("hash");

-- CreateIndex
CREATE UNIQUE INDEX "MediaData_assetObjectId_key" ON "MediaData"("assetObjectId");

-- CreateIndex
CREATE UNIQUE INDEX "FileHandlerTask_assetObjectId_taskType_key" ON "FileHandlerTask"("assetObjectId", "taskType");
