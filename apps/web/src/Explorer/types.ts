export type ExplorerItem = {
  id: number
  name: string
  materializedPath: string
  isDir: boolean
  assetObject: { id: number; hash: string } | null
}
