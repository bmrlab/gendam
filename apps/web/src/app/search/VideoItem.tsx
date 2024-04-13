'use client'
// import Icon from '@muse/ui/icons'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import classNames from 'classnames'
import type { SearchResultPayload } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { formatDuration } from '@/lib/utils'
import Image from 'next/image'
import { useCallback, useEffect, useMemo, useRef } from 'react'

const VideoItem: React.FC<{
  item: SearchResultPayload
  groupFrames: boolean
}> = ({ item, groupFrames }) => {
  const quickViewStore = useQuickViewStore()
  const currentLibrary = useCurrentLibrary()
  const videoRef = useRef<HTMLVideoElement>(null)
  const [frames, width] = useMemo(() => {
    const startTime = Math.floor(item.startTime / 1e3)
    const endTime = Math.floor(item.endTime / 1e3)
    const duration = endTime - startTime
    let repeat = 1;
    let frames = [startTime];
    if (!groupFrames || duration < 1) {
      //
    } else if (duration >= 1 && duration < 6) {
      repeat = 2
      frames = [startTime, endTime]
    } else if (duration >= 6) {
      repeat = 3
      frames = [startTime, Math.floor((startTime + endTime) / 2), endTime]
    }
    const width = repeat * 15 + (repeat - 1) * 1  // gap is 1rem (gap-4 = 1rem)
    return [frames, width]
  }, [groupFrames, item.endTime, item.startTime])

  useEffect(() => {
    const video = videoRef.current
    if (!video) return
    let startTime = Math.max(0, item.startTime / 1e3 - 0.5)
    let endTime = Math.max(startTime, item.endTime / 1e3 + 1.5)
    video.currentTime = startTime
    video.ontimeupdate = () => {
      if (video.currentTime >= endTime) {
        // video.pause();
        // video.ontimeupdate = null;
        video.currentTime = startTime
      }
    }
  }, [item])

  const handleVideoClick = useCallback(
    (item: SearchResultPayload) => {
      quickViewStore.open({
        name: item.name,
        assetObject: {
          id: item.assetObjectId,
          hash: item.assetObjectHash,
        },
        video: {
          currentTime: item.startTime / 1e3,
        },
      })
    },
    [quickViewStore],
  )

  return (
    <div
      className={classNames("group relative overflow-hidden rounded-xl border-4 border-app-line/75")}
      // style={{ minWidth: `${width}rem`, height: '10rem', flex: frames.length }}
      style={{ width: `${width}rem`, height: '10rem' }}
      onClick={() => handleVideoClick(item)}
    >
      <div className="flex items-stretch justify-between h-full">
        {frames.map((frame, index) => (
          <div
            key={index}
            className="visible relative flex-1 cursor-pointer bg-neutral-100"
          >
            <Image
              src={currentLibrary.getThumbnailSrc(item.assetObjectHash, frame)}
              alt={item.name}
              fill={true}
              className="object-cover"
              priority
            ></Image>
          </div>
        ))}
      </div>
      <div
        className={classNames(
          "absolute left-0 top-0 flex h-full w-full flex-col justify-between bg-black/60 px-4 py-2 text-neutral-300",
          "invisible group-hover:visible"
        )}
      >
        <div className="truncate text-xs">
          {item.materializedPath}
          {item.name}
        </div>
        <div className='flex items-center justify-between text-xs'>
          <div>{formatDuration(item.startTime / 1000)}</div>
          <div>→</div>
          <div>{formatDuration(item.endTime / 1000 + 1)}</div>
        </div>
      </div>
    </div>
  )
}

export default VideoItem
