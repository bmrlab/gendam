import { memo, useEffect, useRef } from 'react'
import 'video.js/dist/video-js.css'
import { useVideoPlayer } from './useVideoPlayer'

export const Video = memo(({ hash, currentTime }: { hash: string; currentTime?: number }) => {
  const videoRef = useRef<HTMLVideoElement | null>(null)
  useVideoPlayer(hash, videoRef)

  useEffect(() => {
    if (videoRef.current && currentTime) {
      videoRef.current.currentTime = Math.floor(currentTime / 1e3)
    }
  }, [videoRef.current, currentTime])

  return <video id={hash} ref={videoRef} className="video-js h-full max-h-full w-full max-w-full rounded-md" />
})

Video.displayName = 'Video'
