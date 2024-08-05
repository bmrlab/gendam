import { useCurrentLibrary } from '@/lib/library'
import { formatDuration } from '@/lib/utils'
import classNames from 'classnames'
import Image from 'next/image'
import { useMemo } from 'react'
import { PickSearchResult } from '.'

export default function VideoSearchItem({ data }: { data: PickSearchResult<'video'> }) {
  const currentLibrary = useCurrentLibrary()

  const frames = useMemo(() => {
    const startTime = Math.floor(data.metadata.startTime / 1e3)
    const endTime = Math.floor(data.metadata.endTime / 1e3)
    const duration = endTime - startTime
    if (duration >= 1 && duration < 6) {
      return [startTime, endTime]
    } else if (duration >= 6) {
      return [
        startTime,
        Math.floor((startTime + endTime) / 2),
        endTime,
      ]
    } else {
      return [startTime]
    }
  }, [data])

  return (
    <div className="relative h-full w-full">
      <div className="flex h-full items-stretch justify-between">
        {frames.map((frame, index) => (
          <div key={index} className="visible relative flex-1 bg-neutral-100">
            <Image
              src={currentLibrary.getVideoPreviewSrc(data.filePath.assetObject.hash, frame)}
              alt={data.filePath.name}
              fill={true}
              className="object-cover"
              priority
            ></Image>
          </div>
        ))}
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
