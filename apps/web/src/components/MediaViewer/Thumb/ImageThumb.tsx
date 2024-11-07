import { ExtractExplorerItemWithType } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'

export default function ImageThumb({
  data,
  className,
}: {
  data: ExtractExplorerItemWithType<'Image'>['assetObject']
  className?: string
}) {
  const currentLibrary = useCurrentLibrary()
  return (
    <Image
      src={currentLibrary.getThumbnailSrc(data.hash, 'Image')}
      alt={data.hash}
      fill={true}
      className="object-contain"
      priority
    />
  )
}
