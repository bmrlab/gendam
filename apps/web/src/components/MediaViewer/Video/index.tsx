import { ExtractAssetObject } from '@/Explorer/types'
import 'video.js/dist/video-js.css'
import { useVideoPlayer, VideoPlayerOptions } from './useVideoPlayer'

export function Video({
  assetObject,
  currentTime,
  controls = true,
  autoPlay = false,
  loop = false,
  muted = true,
}: {
  assetObject: ExtractAssetObject<'video'>
} & Partial<VideoPlayerOptions>) {
  const videoRef = useVideoPlayer(assetObject, { currentTime, controls, autoPlay, loop, muted })

  return (
    <div
      id={assetObject.hash}
      ref={videoRef}
      className="h-full max-h-full w-full max-w-full overflow-hidden rounded-md"
    />
  )
}

Video.displayName = 'Video'
