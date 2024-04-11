'use client'
import Icon from '@muse/ui/icons'
import type { SearchResultPayload } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { formatDuration } from '@/lib/utils'
import Image from 'next/image'
import { useEffect, useRef } from 'react'

const VideoItem: React.FC<{
  item: SearchResultPayload
  handleVideoClick: (item: SearchResultPayload) => void
}> = ({ item, handleVideoClick }) => {
  const currentLibrary = useCurrentLibrary()
  const videoRef = useRef<HTMLVideoElement>(null)

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

  return (
    <div
      className="invisible relative w-64 overflow-hidden rounded-md shadow-md hover:visible"
      onClick={() => handleVideoClick(item)}
    >
      <div className="visible relative h-36 w-full cursor-pointer bg-neutral-100">
        {/* <video
          ref={videoRef}
          controls={false}
          autoPlay={false}
          muted
          loop
          style={{ width: '100%', height: '100%', objectFit: 'cover' }}
        >
          <source src={currentLibrary.getFileSrc(item.assetObjectHash)} />
        </video> */}
        <Image
          src={currentLibrary.getThumbnailSrc(item.assetObjectHash, Math.floor(item.startTime / 1e3))}
          alt={item.name}
          fill={true}
          className="object-cover"
          priority
        ></Image>
      </div>
      <div className="absolute left-0 top-0 flex h-full w-full flex-col justify-between bg-black/60 px-4 py-2 text-neutral-300">
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
