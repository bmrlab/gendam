import { ExtractExplorerItemWithType } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'

export default function VideoThumb({
  data,
  className,
}: {
  data: ExtractExplorerItemWithType<'video'>['assetObject']
  className?: string
}) {
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
