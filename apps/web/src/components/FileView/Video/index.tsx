import { ExtractAssetObject } from '@/Explorer/types'
import 'video.js/dist/video-js.css'
import { useVideoPlayer } from './useVideoPlayer'

export function Video({
  assetObject,
  currentTime,
  autoPlay,
}: {
  assetObject: ExtractAssetObject<'video'>
  currentTime?: number
  autoPlay?: boolean
}) {
  const videoRef = useVideoPlayer(assetObject, currentTime, autoPlay)

  return (
    <div
      id={assetObject.hash}
      ref={videoRef}
      className="h-full max-h-full w-full max-w-full overflow-hidden rounded-md"
    />
  )
}

Video.displayName = 'Video'
