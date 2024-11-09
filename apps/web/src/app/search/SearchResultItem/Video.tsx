import { ExtractExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { formatDuration } from '@/lib/utils'
import classNames from 'classnames'
import Image from 'next/image'
import { useMemo } from 'react'

export default function VideoSearchItem({
  assetObject,
  metadata,
  hitReason,
}: ExtractExplorerItem<'SearchResult', 'Video'>) {
  const currentLibrary = useCurrentLibrary()

  const frames = useMemo(() => {
    const startTime = Math.floor(metadata.startTimestamp / 1e3)
    const endTime = Math.floor(metadata.endTimestamp / 1e3)
    const duration = endTime - startTime
    if (duration >= 1 && duration < 6) {
      return [startTime, endTime]
    } else if (duration >= 6) {
      return [startTime, Math.floor((startTime + endTime) / 2), endTime]
    } else {
      return [startTime]
    }
  }, [metadata])

  return (
    <div className="relative h-full w-full">
      <div className="flex h-full items-stretch justify-between">
        {frames.map((frame, index) => (
          <div key={index} className="visible relative flex-1 bg-neutral-100">
            <Image
              src={currentLibrary.getPreviewSrc(assetObject.hash, 'Video', frame)}
              alt={assetObject.hash}
              fill={true}
              className="object-cover"
              priority
            ></Image>
          </div>
        ))}
      </div>
      <div
        className={classNames(
          'absolute left-0 top-0 flex h-full w-full flex-col justify-end bg-black/60 px-4 py-2 text-neutral-300',
          'invisible group-hover:visible',
        )}
      >
        <div className="flex items-center justify-between text-xs">
          <div>{formatDuration(metadata.startTimestamp / 1000)}</div>
          <div>→</div>
          <div>{formatDuration(metadata.endTimestamp / 1000 + 1)}</div>
        </div>
      </div>
    </div>
  )
}
