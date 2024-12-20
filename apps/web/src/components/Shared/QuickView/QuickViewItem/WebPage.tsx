import WebPageViewer from '@/components/MediaViewer/WebPage'
import { ExtractExplorerItemWithType } from '@/Explorer/types'

export default function WebPageQuickView({
  assetObject,
}: {
  assetObject: ExtractExplorerItemWithType<'WebPage'>['assetObject']
}) {
  return (
    <div className="flex h-full w-full items-center justify-center">
      <WebPageViewer hash={assetObject.hash} />
    </div>
  )
}
