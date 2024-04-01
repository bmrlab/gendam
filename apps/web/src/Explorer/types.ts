export type ExplorerItem = {
  id: number
  name: string
  materializedPath: string
  isDir: boolean
  assetObject?: {
    id: number;
    hash: string;
    mediaData?: {
      width: number;
      height: number;
      duration: number;
      bitRate: number;
      size: number;
      mimeType: string;
      hasAudio: boolean;
    } | null
  } | null
  createdAt: string
  updatedAt: string
}

/**
 * FilePath 上面没有 assetObject，主要是 prisma.rs 里面对这个字段设置了 #[specta(skip)]，
 * 但实际返回数据里面这个字段改有的时候还是会有，
 * 这里就先加上 ?，除了 null，还允许 undefined
 */
