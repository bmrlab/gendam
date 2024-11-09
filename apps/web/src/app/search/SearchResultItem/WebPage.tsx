import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'

export default function WebPageSearchItem({ assetObject, hitText }: ExtractExplorerItem<'SearchResult', 'WebPage'>) {
  const currentLibrary = useCurrentLibrary()

  return (
    <div className="h-full w-full">
      <Image
        src={currentLibrary.getThumbnailSrc(assetObject.hash, 'WebPage')}
        alt={assetObject.hash}
        fill={true}
        className="object-cover"
        priority
      />
    </div>
  )
}
