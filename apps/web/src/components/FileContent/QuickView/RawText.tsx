import RawTextViewer from '@/components/FileView/RawText'
import { ExtractExplorerItemWithType } from '@/Explorer/types'

export default function RawTextQuickView({
  assetObject,
}: {
  assetObject: ExtractExplorerItemWithType<'rawText'>['assetObject']
}) {
  return (
    <div className="flex h-full w-full items-center justify-center text-white bg-gray-900 rounded-md">
      <RawTextViewer hash={assetObject.hash} variant='default' />
    </div>
  )
}
