/*
  Warnings:

  - You are about to drop the `MediaData` table. If the table is not empty, all the data it contains will be lost.

*/
-- AlterTable
ALTER TABLE "AssetObject" ADD COLUMN "mediaData" TEXT;

-- DropTable
PRAGMA foreign_keys=off;
DROP TABLE "MediaData";
PRAGMA foreign_keys=on;
