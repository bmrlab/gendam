import { useCurrentLibrary } from '@/lib/library'
import { formatDuration } from '@/lib/utils'
import classNames from 'classnames'
import Image from 'next/image'
import { PickSearchResult } from '.'

export default function AudioSearchItem({ data }: { data: PickSearchResult<'audio'> }) {
  const currentLibrary = useCurrentLibrary()

  return (
    <div className="relative h-full w-full">
      <div className="flex h-full items-stretch justify-between">
        <Image
          src={currentLibrary.getThumbnailSrc(data.filePath.assetObject.hash, 'audio')}
          alt={data.filePath.name}
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
          {data.filePath.materializedPath}
          {data.filePath.name}
        </div>
        <div className="flex items-center justify-between text-xs">
          <div>{formatDuration(data.metadata.startTime / 1000)}</div>
          <div>→</div>
          <div>{formatDuration(data.metadata.endTime / 1000 + 1)}</div>
        </div>
      </div>
    </div>
  )
}
