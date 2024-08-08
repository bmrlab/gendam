import RawTextViewer from '@/components/FileView/RawText'
import { ExtractExplorerItemWithType } from '@/Explorer/types'

export default function RawTextQuickView({
  assetObject,
}: {
  assetObject: ExtractExplorerItemWithType<'rawText'>['assetObject']
}) {
  return (
    <div className="flex h-full w-full items-center justify-center">
      <RawTextViewer hash={assetObject.hash} />
    </div>
  )
}
