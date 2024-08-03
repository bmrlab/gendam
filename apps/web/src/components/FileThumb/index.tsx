'use client'

import { type FilePath } from '@/lib/bindings'
import { AssetObjectType } from '@/lib/library'
import { Document_Light } from '@gendam/assets/images'
import Image from 'next/image'
import { match } from 'ts-pattern'
import AudioThumb from './AudioThumb'
import VideoThumb from './VideoThumb'

export type ThumbnailVariant = 'grid' | 'list' | 'media'

export type PickAssetObject<V extends AssetObjectType> = FilePath['assetObject'] & {
  mediaData: Extract<NonNullable<FilePath['assetObject']>['mediaData'], { contentType: V }>
}

export function matchContentTypePattern<ContentType extends AssetObjectType>(contentType: ContentType) {
  return {
    mediaData: { contentType },
  }
}

export default function FileThumb({
  data,
  className,
  variant,
}: {
  data: NonNullable<FilePath['assetObject']>
  className?: string
  variant: ThumbnailVariant
}) {
  return match(data)
    .with(matchContentTypePattern('Video'), (item) => <VideoThumb data={item} />)
    .with(matchContentTypePattern('Audio'), (item) => <AudioThumb data={item} />)
    .otherwise(() => <Image src={Document_Light} alt="document" fill={true} priority></Image>)
}
