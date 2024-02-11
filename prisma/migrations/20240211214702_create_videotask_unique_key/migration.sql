/*
  Warnings:

  - A unique constraint covering the columns `[videoFileHash,taskType]` on the table `VideoTask` will be added. If there are existing duplicate values, this will fail.

*/
-- CreateIndex
CREATE UNIQUE INDEX `VideoTask_videoFileHash_taskType_key` ON `VideoTask`(`videoFileHash`, `taskType`);
