import Audio from '@/components/FileView/Audio'
import { ExtractExplorerItemWithType } from '@/Explorer/types'

export default function AudioQuickView({
  assetObject,
}: {
  assetObject: ExtractExplorerItemWithType<'audio'>['assetObject']
}) {
  return (
    <div className="flex h-full w-full items-center justify-center">
      <Audio hash={assetObject.hash} duration={assetObject.mediaData?.duration} />
    </div>
  )
}
