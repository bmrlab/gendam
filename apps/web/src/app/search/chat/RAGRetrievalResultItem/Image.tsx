import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import Image from 'next/image'

export default function ImageRetrievalItem({
  assetObject,
  metadata,
  referenceContent,
}: ExtractExplorerItem<'RetrievalResult', 'Image'>) {
  const currentLibrary = useCurrentLibrary()
  // const { data } = rspc.useQuery(['assets.artifacts.image.description', { hash: assetObject.hash }])

  return (
    <div className="flex items-start justify-between space-x-4 rounded-md">
      <div className="flex flex-col space-y-2">
        <div className="relative h-[200px] w-[280px]">
          <Image
            src={currentLibrary.getThumbnailSrc(assetObject.hash, 'Image')}
            className="object-cover"
            fill
            priority
            alt={assetObject.hash}
          />
        </div>
      </div>

      <div className="w-full flex-1">
        <div className="font-semibold">Description</div>
        <div>{referenceContent}</div>
      </div>
    </div>
  )
}
