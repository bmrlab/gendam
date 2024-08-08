'use client'

import { matchExplorerItemWithType } from '@/Explorer/pattern'
import { ExplorerItem } from '@/Explorer/types'
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
  data: ExplorerItem
  className?: string
  variant: ThumbnailVariant
}) {
  return match(data)
    .with(matchExplorerItemWithType('video'), (item) => <VideoThumb data={item.assetObject} className={className} />)
    .with(matchExplorerItemWithType('audio'), (item) => <AudioThumb data={item.assetObject} className={className} />)
    .with(matchExplorerItemWithType('image'), (item) => <ImageThumb data={item.assetObject} className={className} />)
    .with(matchExplorerItemWithType('rawText'), (item) => (
      <RawTextThumb data={item.assetObject} className={className} />
    ))
    .with(matchExplorerItemWithType('webPage'), (item) => (
      <WebPageThumb data={item.assetObject} className={className} />
    ))
    .otherwise(() => <Image src={Document_Light} alt="document" fill={true} priority></Image>)
}
