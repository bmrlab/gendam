export type ExplorerItem = {
  id: number
  name: string
  materializedPath: string
  isDir: boolean
  assetObject: {
    id: number;
    hash: string;
    mediaData: {
      width: number;
      height: number;
      duration: number;
      bitRate: number;
      size: number;
    } | null
  } | null
  createdAt: string
  updatedAt: string
}
