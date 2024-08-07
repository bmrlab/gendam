import { ContentTaskType } from '@/lib/bindings'
import { AssetObjectType } from '@/lib/library'
import { P } from 'ts-pattern'

export function matchFilePath<T extends AssetObjectType>(contentType: T) {
  return {
    type: 'FilePath' as const,
    assetObject: {
      mediaData: {
        contentType,
      },
    },
  }
}

export function matchSearchResult<T extends AssetObjectType>(contentType: T) {
  return {
    ...matchFilePath(contentType),
    type: 'SearchResult' as const,
    metadata: {
      type: contentType,
    },
  }
}

export function matchRetrievalResult<T extends AssetObjectType>(
  contentType: T,
): Omit<ReturnType<typeof matchSearchResult<T>>, 'type'> & { type: 'RetrievalResult'; taskType: { contentType: T } }

export function matchRetrievalResult<
  T extends AssetObjectType,
  V extends Extract<ContentTaskType, { contentType: T }>['taskType'],
>(
  contentType: T,
  taskType: V,
): Omit<ReturnType<typeof matchSearchResult<T>>, 'type'> & {
  type: 'RetrievalResult'
  taskType: { contentType: T; taskType: V }
}

export function matchRetrievalResult<
  T extends AssetObjectType,
  V extends Extract<ContentTaskType, { contentType: T }>['taskType'],
>(contentType: T, taskType?: V) {
  return {
    ...matchSearchResult(contentType),
    type: 'RetrievalResult' as const,
    taskType: {
      contentType,
      taskType: taskType ?? P.any,
    },
  }
}

export function matchExplorerItemWithType<T extends AssetObjectType>(contentType: T) {
  return {
    assetObject: {
      mediaData: {
        contentType,
      },
    },
  }
}
