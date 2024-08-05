import { ExplorerItem } from '@/Explorer/types'
import { FilePath, SearchResultMetadata } from '@/lib/bindings'
import { AssetObjectType } from '@/lib/library'
import { match } from 'ts-pattern'
import { PickAssetObject } from '../FileThumb'
import AudioSearchItem from './Audio'
import VideoSearchItem from './Video'

type SearchResultExplorerItem = Extract<ExplorerItem, { type: 'SearchResult' }>
export type PickSearchResult<V extends AssetObjectType> = SearchResultExplorerItem & {
  metadata: Extract<SearchResultMetadata, { type: V }>
  filePath: FilePath & {
    assetObject: PickAssetObject<V>
  }
}

export function matchSearchResultPattern<SearchResultType extends AssetObjectType>(type: SearchResultType) {
  return {
    metadata: {
      type,
    },
    filePath: {
      assetObject: {
        mediaData: {
          contentType: type,
        },
      },
    },
  }
}

export default function SearchResultItem({ data }: { data: SearchResultExplorerItem }) {
  return match(data)
    .with(matchSearchResultPattern('video'), (item) => <VideoSearchItem data={item} />)
    .with(matchSearchResultPattern('audio'), (item) => <AudioSearchItem data={item} />)
    .otherwise(() => <></>)
}
