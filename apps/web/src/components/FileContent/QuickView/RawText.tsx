import RawTextViewer from '@/components/FileView/RawText'
import { ExtractExplorerItemWithType } from '@/Explorer/types'

export default function RawTextQuickView(props: ExtractExplorerItemWithType<'rawText'>) {
  return (
    <div className="flex h-full w-full items-center justify-center rounded-md bg-gray-900 text-white">
      <RawTextViewer hash={props.assetObject.hash} variant="default" />
    </div>
  )
}
