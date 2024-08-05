import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'
import { PickAssetObject } from '.'

type T = PickAssetObject<'video'>

export default function VideoThumb({ data, className }: { data: T; className?: string }) {
  const currentLibrary = useCurrentLibrary()
  return (
    <Image
      src={currentLibrary.getThumbnailSrc(data.hash, 'video')}
      alt={data.hash}
      fill={true}
      className="object-contain"
      priority
    />
  )
}
