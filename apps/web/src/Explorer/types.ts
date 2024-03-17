export type ExplorerItem = {
  id: number
  name: string
  isDir: boolean
  assetObject: { id: number; hash: string } | null
}
