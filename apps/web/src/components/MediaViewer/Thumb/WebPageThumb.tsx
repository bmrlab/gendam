import { ExtractExplorerItemWithType } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'

export default function WebPageThumb({
  data,
  className,
}: {
  data: ExtractExplorerItemWithType<'WebPage'>['assetObject']
  className?: string
}) {
  const currentLibrary = useCurrentLibrary()
  return (
    <Image
      src={currentLibrary.getThumbnailSrc(data.hash, 'WebPage')}
      alt={data.hash}
      fill={true}
      className="object-contain"
      priority
    />
  )
}
