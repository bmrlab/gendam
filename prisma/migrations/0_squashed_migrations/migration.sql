-- CreateTable
CREATE TABLE "FilePath" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "isDir" BOOLEAN NOT NULL DEFAULT false,
    "materializedPath" TEXT NOT NULL DEFAULT '',
    "relativePath" TEXT,
    "name" TEXT NOT NULL DEFAULT '',
    "description" TEXT,
    "assetObjectId" TEXT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME
);

-- CreateTable
CREATE TABLE "AssetObject" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "hash" TEXT,
    "size" INTEGER,
    "mimeType" TEXT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME
);

-- CreateTable
CREATE TABLE "MediaData" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "width" INTEGER,
    "height" INTEGER,
    "duration" INTEGER,
    "bitRate" INTEGER,
    "hasAudio" BOOLEAN,
    "assetObjectId" TEXT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME
);

-- CreateTable
CREATE TABLE "FileHandlerTask" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "assetObjectId" TEXT NOT NULL,
    "taskType" TEXT NOT NULL,
    "exitCode" INTEGER,
    "exitMessage" TEXT,
    "startsAt" DATETIME,
    "endsAt" DATETIME,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL
);

-- CreateTable
CREATE TABLE "SyncMetadata" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "filePathId" TEXT NOT NULL,
    "subFilePathIds" TEXT,
    "deviceId" TEXT NOT NULL,
    "relativePath" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "lastSyncAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- CreateIndex
CREATE INDEX "FilePath_materializedPath_idx" ON "FilePath"("materializedPath");

-- CreateIndex
CREATE INDEX "FilePath_assetObjectId_idx" ON "FilePath"("assetObjectId");

-- CreateIndex
CREATE INDEX "FilePath_relativePath_idx" ON "FilePath"("relativePath");

-- CreateIndex
CREATE INDEX "MediaData_assetObjectId_idx" ON "MediaData"("assetObjectId");

-- CreateIndex
CREATE UNIQUE INDEX "FileHandlerTask_assetObjectId_taskType_key" ON "FileHandlerTask"("assetObjectId", "taskType");

-- CreateIndex
CREATE UNIQUE INDEX "SyncMetadata_filePathId_key" ON "SyncMetadata"("filePathId");

-- CreateIndex
CREATE INDEX "SyncMetadata_filePathId_deviceId_relativePath_idx" ON "SyncMetadata"("filePathId", "deviceId", "relativePath");
