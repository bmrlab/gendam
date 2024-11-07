import WebPageViewer from '@/components/MediaViewer/WebPage'
import { ExtractExplorerItem } from '@/Explorer/types'
import {
  InspectorItemContainer,
  InspectorItemDivider,
  InspectorItemFilePath,
  InspectorItemMetadata,
  InspectorItemMetadataItem,
  InspectorItemTasks,
  InspectorItemViewer,
} from '.'
import { useSortedTasks } from './hooks'

export default function WebPageDetail({
  filePath,
  assetObject,
}: ExtractExplorerItem<'FilePathWithAssetObject', 'WebPage'>) {
  const { sortedTasks, handleJobsCancel } = useSortedTasks(assetObject)

  return (
    <InspectorItemContainer>
      <InspectorItemViewer>
        <WebPageViewer hash={assetObject.hash} />
      </InspectorItemViewer>

      <InspectorItemFilePath filePath={filePath} />

      <InspectorItemDivider />

      <InspectorItemMetadata data={assetObject}>
        {(assetObject) => (
          <>
            <InspectorItemMetadataItem name="Url">{assetObject.mediaData?.sourceUrl}</InspectorItemMetadataItem>
          </>
        )}
      </InspectorItemMetadata>

      <InspectorItemDivider />

      {/* DEBUG INFO */}
      <div className="text-xs">
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Content Hash</div>
          <div>{assetObject.hash}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Asset Object ID</div>
          <div>{assetObject.id}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Search</div>
          {sortedTasks.some((item) => item.taskType === 'web-page-chunk-sum-embed' && item.exitCode === 0) ? (
            <div className="rounded-full bg-green-100 px-2 text-xs text-green-600">Ready</div>
          ) : (
            <div className="rounded-full bg-orange-100 px-2 text-xs text-orange-600">Not ready</div>
          )}
        </div>
      </div>

      <InspectorItemDivider />

      <InspectorItemTasks sortedTasks={sortedTasks} handleJobsCancel={handleJobsCancel} />
    </InspectorItemContainer>
  )
}
