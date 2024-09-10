import ImageViewer from '@/components/MediaViewer/Image'
import { ExtractExplorerItemWithType } from '@/Explorer/types'

export default function ImageQuickView({
  assetObject,
}: {
  assetObject: ExtractExplorerItemWithType<'image'>['assetObject']
}) {
  return (
    <div className="h-full w-full overflow-hidden">
      <ImageViewer hash={assetObject.hash} mimeType={assetObject.mimeType} />
    </div>
  )
}
