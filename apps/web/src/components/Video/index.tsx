import { useVideoPlayer } from '@/hooks/useVideoPlayer'
import { memo, useRef } from 'react'
import 'video.js/dist/video-js.css'

export const Video = memo(({ hash }: { hash: string }) => {
  const videoRef = useRef<HTMLVideoElement | null>(null)
  useVideoPlayer(hash, videoRef)
  return <video ref={videoRef} className="video-js h-full max-h-full w-full max-w-full rounded-md" />
})

Video.displayName = 'Video'
