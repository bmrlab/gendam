import type { AssetObject, AudioSliceType, ContentIndexMetadata, ContentQueryHitReason, FilePath } from '@/lib/bindings'
import { AssetObjectType } from '@/lib/library'
import { RawTextChunkType, VideoSliceType, WebPageChunkType } from 'api-server/client/types'
import { match } from 'ts-pattern'

// LibraryRoot is a special type of item that represents the root of the library
export type ExplorerItemType =
  | 'LibraryRoot'
  | 'FilePathDir'
  | 'FilePathWithAssetObject'
  | 'SearchResult'
  | 'RetrievalResult'
  | 'Unknown'

// TODO: do not export RawFilePath
export type RawFilePath = Omit<FilePath, 'assetObject'>

type LibraryRootItem = {
  type: 'LibraryRoot'
}

type FilePathDirItem = {
  type: 'FilePathDir'
  filePath: Omit<RawFilePath, 'isDir'> & { isDir: true }
}

type FilePathWithAssetObjectItem = {
  type: 'FilePathWithAssetObject'
  filePath: Omit<RawFilePath, 'isDir'> & { isDir: false }
  assetObject: AssetObject
}

type SearchResultItem = {
  type: 'SearchResult'
  filePaths: RawFilePath[]
  assetObject: AssetObject
  metadata: ContentIndexMetadata
  hitReason: ContentQueryHitReason
}

type RetrievalResultItem = {
  type: 'RetrievalResult'
  assetObject: AssetObject
  metadata: ContentIndexMetadata
  referenceContent: string
}

type UnknownItem = {
  type: 'Unknown'
}

export type ExplorerItem =
  | LibraryRootItem
  | FilePathDirItem
  | FilePathWithAssetObjectItem
  | SearchResultItem
  | RetrievalResultItem
  | UnknownItem

export type ExtractAssetObject<V extends AssetObjectType> = AssetObject & {
  mediaData: Extract<AssetObject['mediaData'], { contentType: V }> | null
}

export type ExtractFilePathWithAssetObjectItem<V extends AssetObjectType> = FilePathWithAssetObjectItem & {
  assetObject: ExtractAssetObject<V>
}

export type ExtractSearchResultItem<V extends AssetObjectType> = SearchResultItem & {
  assetObject: ExtractAssetObject<V>
  metadata: ContentIndexMetadata & { contentType: V }
}

export type ValidMetadataType<V extends AssetObjectType = AssetObjectType> = V extends 'Video'
  ? { sliceType: VideoSliceType }
  : V extends 'Audio'
    ? { sliceType: AudioSliceType }
    : V extends 'WebPage'
      ? { chunkType: WebPageChunkType }
      : V extends 'RawText'
        ? { chunkType: RawTextChunkType }
        : {}

export type ExtractRetrievalResultItem<
  V extends AssetObjectType = AssetObjectType,
  U extends ValidMetadataType<V> = ValidMetadataType<V>,
> = RetrievalResultItem & {
  assetObject: ExtractAssetObject<V>
  metadata: ContentIndexMetadata & { contentType: V } & U
}

export type ExtractExplorerItem<
  T extends ExplorerItemType =
    | 'LibraryRoot'
    | 'FilePathDir'
    | 'FilePathWithAssetObject'
    | 'SearchResult'
    | 'RetrievalResult'
    | 'Unknown',
  V extends AssetObjectType = AssetObjectType,
  U extends ValidMetadataType<V> = ValidMetadataType<V>,
> = T extends 'LibraryRoot'
  ? LibraryRootItem
  : T extends 'FilePathDir'
    ? FilePathDirItem
    : T extends 'FilePathWithAssetObject'
      ? ExtractFilePathWithAssetObjectItem<V>
      : T extends 'SearchResult'
        ? ExtractSearchResultItem<V>
        : T extends 'RetrievalResult'
          ? // ExtractRetrievalResultItem<V, U>
            // 这里其实就是 ExtractRetrievalResultItem<V, U>，但是得展开来写，不然类型会报错
            RetrievalResultItem & {
              assetObject: ExtractAssetObject<V>
              metadata: ContentIndexMetadata & { contentType: V } & U
            }
          : UnknownItem

export type ExtractExplorerItemWithType<V extends AssetObjectType = AssetObjectType> =
  | ExtractFilePathWithAssetObjectItem<V>
  | ExtractSearchResultItem<V>
  | ExtractRetrievalResultItem<V>

export function uniqueId(item: ExplorerItem): string {
  switch (item.type) {
    case 'LibraryRoot':
      return 'LibraryRoot'
    case 'FilePathDir':
      return `FilePathDir:${item.filePath.id}`
    case 'FilePathWithAssetObject':
      return `FilePathWithAssetObject:${item.filePath.id}`
    case 'SearchResult':
      return `SearchResult:${item.assetObject.id}:${uniqueIdForSearchMetadata(item.metadata)}`
    case 'RetrievalResult':
      return `RetrievalResult:${item.assetObject.id}:${uniqueIdForSearchMetadata(item.metadata)}`
    case 'Unknown':
      return `Unknown:${Math.random()}`
  }
}

function uniqueIdForSearchMetadata(item: ContentIndexMetadata): string {
  return match(item)
    .with({ contentType: 'Video' }, (item) => `${item.contentType}:${item.sliceType}:${item.startTimestamp}`)
    .with({ contentType: 'Audio' }, (item) => `${item.contentType}:${item.sliceType}:${item.startTimestamp}`)
    .with({ contentType: 'Image' }, (item) => `${item.contentType}`)
    .with({ contentType: 'RawText' }, (item) => `${item.contentType}:${item.chunkType}:${item.startIndex}`)
    .with({ contentType: 'WebPage' }, (item) => `${item.contentType}:${item.chunkType}:${item.startIndex}`)
    .exhaustive()
}
