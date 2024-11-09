import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
// import { rspc } from '@/lib/rspc'
// import classNames from 'classnames'
import Image from 'next/image'

export default function ImageSearchItem({ assetObject, hitReason }: ExtractExplorerItem<'SearchResult', 'Image'>) {
  const currentLibrary = useCurrentLibrary()
  // const { data: description } = rspc.useQuery(['assets.artifacts.image.description', { hash: assetObject.hash }])

  return (
    <div className="relative h-full w-full">
      <Image
        src={currentLibrary.getThumbnailSrc(assetObject.hash, 'Image')}
        alt={assetObject.hash}
        fill={true}
        className="object-cover"
        priority
      />

      {/* {description && (
        <div
          className={classNames(
            'absolute left-0 top-0 flex h-full w-full flex-col justify-end bg-black/60 px-4 py-2 text-neutral-300',
            'invisible group-hover:visible',
            'overflow-scroll',
          )}
        >
          <div className="line-clamp-3 text-xs">{description}</div>
        </div>
      )} */}
    </div>
  )
}
