-- CreateTable
CREATE TABLE "VideoTask" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "videoPath" TEXT NOT NULL,
    "videoFileHash" TEXT NOT NULL,
    "taskType" TEXT NOT NULL,
    "startsAt" DATETIME,
    "endsAt" DATETIME,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL
);

-- CreateIndex
CREATE UNIQUE INDEX "VideoTask_videoFileHash_taskType_key" ON "VideoTask"("videoFileHash", "taskType");
