-- CreateTable
CREATE TABLE "SyncMetadata" (
    "id" TEXT NOT NULL PRIMARY KEY,
    "filePathId" TEXT NOT NULL,
    "deviceId" TEXT NOT NULL,
    "relativePath" TEXT NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "lastSyncAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- CreateIndex
CREATE UNIQUE INDEX "SyncMetadata_filePathId_key" ON "SyncMetadata"("filePathId");

-- CreateIndex
CREATE INDEX "SyncMetadata_filePathId_deviceId_relativePath_idx" ON "SyncMetadata"("filePathId", "deviceId", "relativePath");
