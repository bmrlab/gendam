import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'
import { PickAssetObject } from '.'

type T = PickAssetObject<'Video'>

export default function VideoThumb({ data, className }: { data: T; className?: string }) {
  const currentLibrary = useCurrentLibrary()
  return (
    <Image
      src={currentLibrary.getThumbnailSrc(data.hash, 'Video')}
      alt={data.hash}
      fill={true}
      className="object-contain"
      priority
    />
  )
}
