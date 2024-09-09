import { Video } from '@/components/FileView/Video'
import { ExtractExplorerItemWithType } from '@/Explorer/types'

export default function VideoQuickView(props: ExtractExplorerItemWithType<'video'>) {
  return (
    <div className="flex h-full w-full items-center justify-center">
      <Video
        assetObject={props.assetObject}
        currentTime={'metadata' in props ? props.metadata.startTime : void 0}
        autoPlay
      />
    </div>
  )
}
