/*
  Warnings:

  - A unique constraint covering the columns `[docId]` on the table `FilePath` will be added. If there are existing duplicate values, this will fail.

*/
-- AlterTable
ALTER TABLE "FilePath" ADD COLUMN "docId" TEXT;

-- CreateTable
CREATE TABLE "Sync" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    "docId" TEXT NOT NULL,
    "doc" BLOB NOT NULL
);

-- CreateIndex
CREATE UNIQUE INDEX "Sync_docId_key" ON "Sync"("docId");

-- CreateIndex
CREATE INDEX "Sync_docId_idx" ON "Sync"("docId");

-- CreateIndex
CREATE UNIQUE INDEX "FilePath_docId_key" ON "FilePath"("docId");
