/*
  Warnings:

  - You are about to drop the `VideoTask` table. If the table is not empty, all the data it contains will be lost.

*/
-- AlterTable
ALTER TABLE "AssetObject" ADD COLUMN "hash" TEXT;

-- DropTable
PRAGMA foreign_keys=off;
DROP TABLE "VideoTask";
PRAGMA foreign_keys=on;

-- CreateTable
CREATE TABLE "FileHandlerTask" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "asssetObjectId" INTEGER NOT NULL,
    "taskType" TEXT NOT NULL,
    "startsAt" DATETIME,
    "endsAt" DATETIME,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "FileHandlerTask_asssetObjectId_fkey" FOREIGN KEY ("asssetObjectId") REFERENCES "AssetObject" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateIndex
CREATE UNIQUE INDEX "FileHandlerTask_asssetObjectId_taskType_key" ON "FileHandlerTask"("asssetObjectId", "taskType");
