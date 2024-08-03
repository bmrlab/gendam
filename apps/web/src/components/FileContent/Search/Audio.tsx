import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { formatDuration } from '@/lib/utils'
import classNames from 'classnames'
import Image from 'next/image'

export default function AudioSearchItem({ assetObject, metadata }: ExtractExplorerItem<'SearchResult','audio'>) {
  const currentLibrary = useCurrentLibrary()

  return (
    <div className="relative h-full w-full">
      <div className="flex h-full items-stretch justify-between">
        <Image
          src={currentLibrary.getThumbnailSrc(assetObject.hash, 'audio')}
          alt={assetObject.hash}
          fill={true}
          className="object-cover"
          priority
        />
      </div>
      <div
        className={classNames(
          'absolute left-0 top-0 flex h-full w-full flex-col justify-between bg-black/60 px-4 py-2 text-neutral-300',
          'invisible group-hover:visible',
        )}
      >
        <div className="truncate text-xs">
          {/* {filePath.materializedPath}
          {filePath.name} */}
          {assetObject.hash}
        </div>
        <div className="flex items-center justify-between text-xs">
          <div>{formatDuration(metadata.startTime / 1000)}</div>
          <div>→</div>
          <div>{formatDuration(metadata.endTime / 1000 + 1)}</div>
        </div>
      </div>
    </div>
  )
}
