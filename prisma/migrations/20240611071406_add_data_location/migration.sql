-- CreateTable
CREATE TABLE "DataLocation" (
    "id" INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    "medium" TEXT NOT NULL,
    "config" TEXT NOT NULL DEFAULT '{}',
    "assetObjectId" INTEGER NOT NULL,
    "createdAt" DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    "updatedAt" DATETIME NOT NULL,
    CONSTRAINT "DataLocation_assetObjectId_fkey" FOREIGN KEY ("assetObjectId") REFERENCES "AssetObject" ("id") ON DELETE CASCADE ON UPDATE CASCADE
);

-- CreateIndex
CREATE UNIQUE INDEX "DataLocation_assetObjectId_medium_key" ON "DataLocation"("assetObjectId", "medium");
