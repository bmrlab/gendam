/*
  Warnings:

  - Added the required column `method` to the `VideoFrameCaption` table without a default value. This is not possible if the table is not empty.

*/
-- AlterTable
ALTER TABLE "MediaData" ADD COLUMN "hasAudio" BOOLEAN;

-- RedefineTables
PRAGMA foreign_keys=OFF;
CREATE TABLE "new_VideoFrameCaption" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "caption" TEXT NOT NULL,
    "method" TEXT NOT NULL,
    "videoFrameId" INTEGER NOT NULL,
    CONSTRAINT "VideoFrameCaption_videoFrameId_fkey" FOREIGN KEY ("videoFrameId") REFERENCES "VideoFrame" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);
INSERT INTO "new_VideoFrameCaption" ("caption", "createdAt", "id", "updatedAt", "videoFrameId") SELECT "caption", "createdAt", "id", "updatedAt", "videoFrameId" FROM "VideoFrameCaption";
DROP TABLE "VideoFrameCaption";
ALTER TABLE "new_VideoFrameCaption" RENAME TO "VideoFrameCaption";
CREATE UNIQUE INDEX "VideoFrameCaption_videoFrameId_method_key" ON "VideoFrameCaption"("videoFrameId", "method");
PRAGMA foreign_key_check;
PRAGMA foreign_keys=ON;
