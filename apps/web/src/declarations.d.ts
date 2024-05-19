/**
 * @todo rename '@/lib/bindings' to '@gendam/client'
 */
declare module '@/lib/bindings' {

  import { AssetObject as AssetObjectSimple, FilePath as FilePathSimple, MediaData } from 'api-server/client/types'
  export type * from 'api-server/client/types'

  export type AssetObject = AssetObjectSimple & {
    mediaData?: MediaData | null
  }

  export type FilePath = FilePathSimple & {
    assetObject?: AssetObject | null
  }

  /**
   * FilePath 上面没有 assetObject，主要是 prisma.rs 里面对这个字段设置了 #[specta(skip)]，
   * 但实际返回数据里面这个字段改有的时候还是会有，
   * 这里就先加上 ?，除了 null，还允许 undefined
   */

}
