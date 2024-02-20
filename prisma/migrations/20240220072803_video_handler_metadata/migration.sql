-- CreateTable
CREATE TABLE "VideoFrame" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "fileIdentifier" TEXT NOT NULL,
    "timestamp" INTEGER NOT NULL
);

-- CreateTable
CREATE TABLE "VideoTranscript" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "fileIdentifier" TEXT NOT NULL,
    "startTimestamp" INTEGER NOT NULL,
    "endTimestamp" INTEGER NOT NULL,
    "text" TEXT NOT NULL
);

-- CreateTable
CREATE TABLE "VideoFrameCaption" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "caption" TEXT NOT NULL,
    "videoFrameId" INTEGER NOT NULL,
    CONSTRAINT "VideoFrameCaption_videoFrameId_fkey" FOREIGN KEY ("videoFrameId") REFERENCES "VideoFrame" ("id") ON DELETE RESTRICT ON UPDATE CASCADE
);

-- CreateIndex
CREATE UNIQUE INDEX "VideoFrame_fileIdentifier_timestamp_key" ON "VideoFrame"("fileIdentifier", "timestamp");

-- CreateIndex
CREATE UNIQUE INDEX "VideoTranscript_fileIdentifier_startTimestamp_endTimestamp_key" ON "VideoTranscript"("fileIdentifier", "startTimestamp", "endTimestamp");

-- CreateIndex
CREATE UNIQUE INDEX "VideoFrameCaption_videoFrameId_key" ON "VideoFrameCaption"("videoFrameId");
