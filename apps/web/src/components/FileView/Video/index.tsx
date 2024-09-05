import { ExtractAssetObject } from '@/Explorer/types'
import 'video.js/dist/video-js.css'
import { useVideoPlayer } from './useVideoPlayer'

export function Video({
  assetObject,
  currentTime,
}: {
  assetObject: ExtractAssetObject<'video'>
  currentTime?: number
}) {
  const videoRef = useVideoPlayer(assetObject, currentTime)

  return (
    <div
      id={assetObject.hash}
      ref={videoRef}
      className="h-full max-h-full w-full max-w-full overflow-hidden rounded-md"
    />
  )
}

Video.displayName = 'Video'
