'use client'

import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'
import { PickAssetObject } from '.'

type T = PickAssetObject<'audio'>

export default function AudioThumb({ data, className }: { data: T; className?: string }) {
  const currentLibrary = useCurrentLibrary()
  return (
    <Image
      src={currentLibrary.getThumbnailSrc(data.hash, 'audio')}
      alt={data.hash}
      fill={true}
      className="object-contain"
      priority
    />
  )
}
