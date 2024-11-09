import { AssetObjectType } from '@/lib/library'
import { ContentIndexMetadata } from 'api-server/client/types'
import { P } from 'ts-pattern'
import { ValidMetadataType } from './types'

export function matchFilePath<T extends AssetObjectType>(contentType: T) {
  return {
    type: 'FilePathWithAssetObject' as const,
    assetObject: {
      mediaData: {
        contentType,
      },
    },
  }
}

// filter with assetObject.mediaData.contentType on search result
export function matchSearchResult<T extends AssetObjectType>(contentType: T) {
  return {
    ...matchFilePath(contentType),
    type: 'SearchResult' as const,
    metadata: { contentType },
  }
}

export function matchRetrievalResult<T extends AssetObjectType>(
  contentType: T,
): Omit<ReturnType<typeof matchSearchResult<T>>, 'type'> & {
  type: 'RetrievalResult'
}

export function matchRetrievalResult<T extends AssetObjectType, U extends ValidMetadataType<T>>(
  contentType: T,
  metadataType: U,
): Omit<ReturnType<typeof matchSearchResult<T>>, 'type'> & {
  type: 'RetrievalResult'
  metadata: ContentIndexMetadata & { contentType: T } & U
}

// filter with assetObject.mediaData.contentType and content specific metadata on retrieval result
export function matchRetrievalResult<T extends AssetObjectType, U extends ValidMetadataType<T>>(
  contentType: T,
  metadata?: U,
) {
  return {
    ...matchSearchResult(contentType),
    type: 'RetrievalResult' as const,
    metadata: metadata ?? P.any,
  }
}

export function matchExplorerItemWithType<T extends AssetObjectType>(contentType: T) {
  return P.union(matchFilePath(contentType), matchSearchResult(contentType), matchRetrievalResult(contentType))
}
