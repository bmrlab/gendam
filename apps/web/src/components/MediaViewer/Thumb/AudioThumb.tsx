import { ExtractExplorerItemWithType } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'

export default function AudioThumb({
  data,
  className,
}: {
  data: ExtractExplorerItemWithType<'audio'>['assetObject']
  className?: string
}) {
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
