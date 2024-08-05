import {
  ContentTaskTypeSpecta,
  FilePathWithAssetObjectData,
  RetrievalResultPayload,
  SearchResultMetadata,
} from '@/lib/bindings'
import { AssetObjectType } from '@/lib/library'
import { match } from 'ts-pattern'
import { PickAssetObject } from '../FileThumb'
import AudioRetrievalItem from './Audio'
import VideoRetrievalItem from './Video'

export type PickRetrievalResult<V extends AssetObjectType> = RetrievalResultPayload & {
  filePath: FilePathWithAssetObjectData & {
    assetObject: PickAssetObject<V>
  }
  metadata: Extract<SearchResultMetadata, { type: V }>
  taskType: Extract<ContentTaskTypeSpecta, { contentType: V }>
}

export type PickRetrievalResultWithTask<
  V extends AssetObjectType,
  U extends Extract<ContentTaskTypeSpecta, { contentType: V }>['taskType'],
> = PickRetrievalResult<V> & {
  taskType: { contentType: V; taskType: U }
}

export function matchRetrievalResultPattern<RetrievalResultType extends AssetObjectType>(type: RetrievalResultType) {
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
    taskType: {
      contentType: type,
    },
  }
}

export function matchRetrievalResultWithTaskPattern<RetrievalResultType extends AssetObjectType, RetrievalTaskType extends Extract<ContentTaskTypeSpecta, { contentType: RetrievalResultType }>['taskType']>(contentType: RetrievalResultType, taskType: RetrievalTaskType) {
  return {
    metadata: {
      type: contentType,
    },
    filePath: {
      assetObject: {
        mediaData: {
          contentType,
        },
      },
    },
    taskType: {
      contentType,
      taskType,
    },
  }
}

export default function RetrievalResultItem({ data }: { data: RetrievalResultPayload }) {
  return match(data)
    .with(matchRetrievalResultPattern('video'), (item) => <VideoRetrievalItem data={item} />)
    .with(matchRetrievalResultPattern('audio'), (item) => <AudioRetrievalItem data={item} />)
    .otherwise(() => <></>)
}
