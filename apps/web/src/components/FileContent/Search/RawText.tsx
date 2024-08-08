import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { formatDuration } from '@/lib/utils'
import { Document_Light } from '@gendam/assets/images'
import classNames from 'classnames'
import Image from 'next/image'

export default function RawTextSearchItem({ assetObject, metadata }: ExtractExplorerItem<'SearchResult', 'rawText'>) {
  const currentLibrary = useCurrentLibrary()

  return (
    <div className="relative h-full w-full">
      <div className="flex h-full items-stretch justify-between">
        <Image
          src={Document_Light}
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
        <div className="truncate text-xs">{assetObject.hash}</div>
        <div className="flex items-center justify-between text-xs">
          <div>{metadata.index}</div>
        </div>
      </div>
    </div>
  )
}
