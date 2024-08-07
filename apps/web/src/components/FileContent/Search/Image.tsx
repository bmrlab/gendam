import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'

export default function ImageSearchItem({ assetObject }: ExtractExplorerItem<'SearchResult', 'image'>) {
  const currentLibrary = useCurrentLibrary()

  return (
    <div className="h-full w-full">
      <Image
        src={currentLibrary.getThumbnailSrc(assetObject.hash, 'image')}
        alt={assetObject.hash}
        fill={true}
        className="object-cover"
        priority
      />
    </div>
  )
}
