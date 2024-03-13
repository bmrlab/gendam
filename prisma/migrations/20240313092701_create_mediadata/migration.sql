-- CreateTable
CREATE TABLE "MediaData" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "width" INTEGER,
    "height" INTEGER,
    "duration" INTEGER,
    "bitRate" INTEGER,
    "description" TEXT,
    "asssetObjectId" INTEGER NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "MediaData_asssetObjectId_fkey" FOREIGN KEY ("asssetObjectId") REFERENCES "AssetObject" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateIndex
CREATE UNIQUE INDEX "MediaData_asssetObjectId_key" ON "MediaData"("asssetObjectId");
