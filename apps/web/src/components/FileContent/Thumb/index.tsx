'use client'

import { matchExplorerItemWithType } from '@/Explorer/pattern'
import { ExplorerItem } from '@/Explorer/types'
import { Document_Light } from '@gendam/assets/images'
import Image from 'next/image'
import { match } from 'ts-pattern'
import AudioThumb from './AudioThumb'
import VideoThumb from './VideoThumb'
import { ThumbnailVariant } from './types'

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
    .with(matchExplorerItemWithType('video'), (item) => <VideoThumb data={item.assetObject} />)
    .with(matchExplorerItemWithType('audio'), (item) => <AudioThumb data={item.assetObject} />)
    .otherwise(() => <Image src={Document_Light} alt="document" fill={true} priority></Image>)
}
