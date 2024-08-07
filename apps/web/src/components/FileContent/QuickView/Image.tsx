import ImageViewer from '@/components/FileView/Image'
import { ExtractExplorerItemWithType } from '@/Explorer/types'

export default function ImageQuickView({
  assetObject,
}: {
  assetObject: ExtractExplorerItemWithType<'image'>['assetObject']
}) {
  return (
    <div className="flex h-full w-full items-center justify-center">
      <ImageViewer hash={assetObject.hash} />
    </div>
  )
}
