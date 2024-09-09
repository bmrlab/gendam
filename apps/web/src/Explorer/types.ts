import type { AssetObject, ContentTaskType, FilePath, SearchResultMetadata } from '@/lib/bindings'
import { AssetObjectType } from '@/lib/library'
import { match } from 'ts-pattern'

export type ExplorerItemType = 'FilePath' | 'SearchResult' | 'RetrievalResult' | 'Unknown'
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

export type ExplorerItem = FilePathItem | SearchResultItem | RetrievalResultItem | UnknownItem

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

export type ExtractFilePathItem<T extends AssetObjectType> = FilePathItem & ExtractBaseItem<T>

export type ExtractSearchResultItem<T extends AssetObjectType> = SearchResultItem & ExtractBaseSearchResultItem<T>
export type ExtractRetrievalResultItem<
  T extends AssetObjectType,
  V extends ValidContentTaskType<T> = ValidContentTaskType<T>,
> = RetrievalResultItem &
  ExtractBaseSearchResultItem<T> & {
    taskType: { contentType: T; taskType: V }
  }

export type ExtractExplorerItem<
  T extends ExplorerItemType = 'FilePath' | 'SearchResult' | 'RetrievalResult',
  V extends AssetObjectType = AssetObjectType,
  U extends ValidContentTaskType<V> = ValidContentTaskType<V>,
> = T extends 'Unknown'
  ? UnknownItem
  : T extends 'FilePath'
    ? ExtractFilePathItem<V>
    : T extends 'SearchResult'
      ? ExtractSearchResultItem<V>
      : RetrievalResultItem &
          ExtractBaseSearchResultItem<V> & {
            taskType: { contentType: V; taskType: U }
          }

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
