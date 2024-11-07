import { Video } from '@/components/MediaViewer/Video'
import { ExtractExplorerItemWithType } from '@/Explorer/types'

export default function VideoQuickView(props: ExtractExplorerItemWithType<'Video'>) {
  return (
    <div className="flex h-full w-full items-center justify-center">
      <Video
        assetObject={props.assetObject}
        currentTime={'metadata' in props ? props.metadata.startTimestamp : void 0}
        autoPlay={true}
        muted={false}
      />
    </div>
  )
}
