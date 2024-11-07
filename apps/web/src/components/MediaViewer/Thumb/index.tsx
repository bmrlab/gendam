'use client'

import { matchExplorerItemWithType } from '@/Explorer/pattern'
import { ExtractExplorerItem } from '@/Explorer/types'
import { Document_Light } from '@gendam/assets/images'
import Image from 'next/image'
import { match } from 'ts-pattern'
import AudioThumb from './AudioThumb'
import ImageThumb from './ImageThumb'
import RawTextThumb from './RawTextThumb'
import { ThumbnailVariant } from './types'
import VideoThumb from './VideoThumb'
import WebPageThumb from './WebPageThumb'

export default function FileThumb({
  data,
  className,
  variant,
}: {
  data: ExtractExplorerItem<'FilePathWithAssetObject'>
  className?: string
  variant: ThumbnailVariant
}) {
  return match(data)
    .with(matchExplorerItemWithType('Video'), (item) => <VideoThumb data={item.assetObject} className={className} />)
    .with(matchExplorerItemWithType('Audio'), (item) => <AudioThumb data={item.assetObject} className={className} />)
    .with(matchExplorerItemWithType('Image'), (item) => <ImageThumb data={item.assetObject} className={className} />)
    .with(matchExplorerItemWithType('RawText'), (item) => (
      <RawTextThumb data={item.assetObject} className={className} />
    ))
    .with(matchExplorerItemWithType('WebPage'), (item) => (
      <WebPageThumb data={item.assetObject} className={className} />
    ))
    .otherwise(() => (
      <Image src={Document_Light} alt="document" fill={true} className="object-contain" priority></Image>
    ))
}
