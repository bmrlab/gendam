/**
 * @todo rename '@/lib/bindings' to '@gendam/client'
 */
declare module '@/lib/bindings' {
  import type * as T from 'api-server/client/types'
  export type * from 'api-server/client/types'

  export type AssetObject = T.AssetObject & {
    mediaData?: T.MediaData | null
  }

  export type FilePath = T.FilePath & {
    assetObject?: AssetObject | null
  }

  export type SearchResultPayload = Omit<T.SearchResultPayload, 'filePath'> & {
    filePath: FilePath
  }

  /**
   * FilePath 上面没有 assetObject，主要是 prisma.rs 里面对这个字段设置了 #[specta(skip)]，
   * 但实际返回数据里面这个字段改有的时候还是会有，
   * 这里就先加上 ?，除了 null，还允许 undefined
   */
}

/*
  TypeScript declaration module for mux.js
*/
declare module 'mux.js' {
  namespace mp4 {
    class Transmuxer {
      constructor(options?: TransmuxerOptions)
      on(event: string, callback: (data?: any) => void): void
      off(event: string): void
      push(data: Uint8Array): void
      flush(): void
      reset(): void
      endTimeline(): void
    }

    interface TransmuxerOptions {
      // todo
    }

    namespace tools {
      function inspect(bytes: Uint8Array): any // todo
    }
  }
}
