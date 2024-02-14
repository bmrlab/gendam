/*
  Warnings:

  - The values [FRAMES] on the enum `VideoTask_taskType` will be removed. If these variants are still used in the database, this will fail.

*/
-- AlterTable
ALTER TABLE `VideoTask` MODIFY `taskType` ENUM('FRAME', 'FRAME_CAPTION', 'FRAME_CONTENT_EMBEDDING', 'AUDIO', 'TRANSCRIPT', 'TRANSCRIPT_EMBEDDING') NOT NULL;
