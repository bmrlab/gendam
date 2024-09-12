import type { AssetObject, ContentTaskType, FilePath, SearchResultMetadata } from '@/lib/bindings'
import { AssetObjectType } from '@/lib/library'
import { match } from 'ts-pattern'

// LibraryRoot is a special type of item that represents the root of the library
export type ExplorerItemType = 'FilePath' | 'SearchResult' | 'RetrievalResult' | 'LibraryRoot' | 'Unknown'
export type RawFilePath = Omit<FilePath, 'assetObject'>

type BaseItem = {
  assetObject: AssetObject
}

type BaseSearchResultItem = BaseItem & {
  metadata: SearchResultMetadata
}

type FilePathItem = Partial<BaseItem> & {
  type: 'FilePath'
  filePath: RawFilePath
}

type SearchResultItem = BaseSearchResultItem & {
  type: 'SearchResult'
  filePaths: RawFilePath[]
}

type RetrievalResultItem = BaseSearchResultItem & {
  type: 'RetrievalResult'
  taskType: ContentTaskType
}

type UnknownItem = {
  type: 'Unknown'
}

type LibraryRootItem = {
  type: 'LibraryRoot'
}

export type ExplorerItem = FilePathItem | SearchResultItem | RetrievalResultItem | LibraryRootItem | UnknownItem

type ValidContentTaskType<T extends AssetObjectType = AssetObjectType> = Extract<
  ContentTaskType,
  { contentType: T }
>['taskType']

export type ExtractAssetObject<T extends AssetObjectType> = AssetObject & {
  mediaData: Extract<AssetObject['mediaData'], { contentType: T }> | null
}
type ExtractBaseItem<T extends AssetObjectType> = BaseItem & { assetObject: ExtractAssetObject<T> }

type ExtractBaseSearchResultItem<T extends AssetObjectType> = ExtractBaseItem<T> & {
  metadata: SearchResultMetadata & { type: T }
}

// export type ExtractFilePathItem<T extends AssetObjectType> = FilePathItem & ExtractBaseItem<T>
export type ExtractFilePathItem<T extends AssetObjectType> = FilePathItem & {
  assetObject?: ExtractAssetObject<T>
}

export type ExtractSearchResultItem<T extends AssetObjectType> = SearchResultItem & ExtractBaseSearchResultItem<T>
export type ExtractRetrievalResultItem<
  T extends AssetObjectType,
  V extends ValidContentTaskType<T> = ValidContentTaskType<T>,
> = RetrievalResultItem &
  ExtractBaseSearchResultItem<T> & {
    taskType: { contentType: T; taskType: V }
  }

export type ExtractExplorerItem<
  T extends ExplorerItemType = 'FilePath' | 'SearchResult' | 'RetrievalResult' | 'LibraryRoot' | 'Unknown',
  V extends AssetObjectType = AssetObjectType,
  U extends ValidContentTaskType<V> = ValidContentTaskType<V>,
> = T extends 'FilePath'
  ? ExtractFilePathItem<V>
  : T extends 'SearchResult'
    ? ExtractSearchResultItem<V>
    : T extends 'RetrievalResult'
      ? RetrievalResultItem &
          ExtractBaseSearchResultItem<V> & {
            taskType: { contentType: V; taskType: U }
          }
      : T extends 'LibraryRoot'
        ? LibraryRootItem
        : UnknownItem

export type ExtractExplorerItemWithType<T extends AssetObjectType = AssetObjectType> =
  | ExtractFilePathItem<T>
  | ExtractSearchResultItem<T>
  | ExtractRetrievalResultItem<T>

export function uniqueId(item: ExplorerItem): string {
  switch (item.type) {
    case 'FilePath':
      return `FilePath:${item.filePath.id}`
    case 'SearchResult':
      return `SearchResult:${item.assetObject.id}:${uniqueIdForSearchMetadata(item.metadata)}`
    case 'RetrievalResult':
      return `RetrievalResult:${item.assetObject.id}:${item.taskType}:${uniqueIdForSearchMetadata(item.metadata)}`
    case 'LibraryRoot':
      return 'LibraryRoot'
    case 'Unknown':
      return `Unknown:${Math.random()}`
  }
}

function uniqueIdForSearchMetadata(item: SearchResultMetadata): string {
  return match(item)
    .with({ type: 'video' }, (item) => item.startTime.toString())
    .with({ type: 'audio' }, (item) => item.startTime.toString())
    .with({ type: 'image' }, (item) => item.type)
    .with({ type: 'rawText' }, (item) => item.startIndex.toString())
    .with({ type: 'webPage' }, (item) => item.startIndex.toString())
    .exhaustive()
}
