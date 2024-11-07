import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'

export default function WebPageRetrievalItem({
  assetObject,
  metadata,
}: ExtractExplorerItem<'RetrievalResult', 'WebPage'>) {
  const currentLibrary = useCurrentLibrary()

  return (
    <div className="bg-app-overlay flex flex-col space-y-2 rounded-md p-2">
      <div className="relative h-40 w-64">
        <Image
          src={currentLibrary.getThumbnailSrc(assetObject.hash, 'WebPage')}
          className="object-cover"
          fill
          priority
          alt={assetObject.hash}
        />
      </div>
    </div>
  )
}
