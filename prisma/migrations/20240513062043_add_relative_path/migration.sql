-- AlterTable
ALTER TABLE "FilePath" ADD COLUMN "relativePath" TEXT;

-- CreateIndex
CREATE INDEX "FilePath_relativePath_idx" ON "FilePath"("relativePath");
