import RawTextViewer from '@/components/MediaViewer/RawText'
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

export default function RawTextDetail({
  filePath,
  assetObject,
}: ExtractExplorerItem<'FilePathWithAssetObject', 'rawText'>) {
  const { sortedTasks, handleJobsCancel } = useSortedTasks(assetObject)

  return (
    <InspectorItemContainer>
      <InspectorItemViewer>
        <RawTextViewer hash={assetObject.hash} variant="sm" />
      </InspectorItemViewer>

      <InspectorItemFilePath filePath={filePath} />

      <InspectorItemDivider />

      <InspectorItemMetadata data={assetObject}>
        {(assetObject) => (
          <>
            <InspectorItemMetadataItem name="Text Count">
              {assetObject.mediaData?.text_count ?? 0}
            </InspectorItemMetadataItem>
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
          {sortedTasks.some((item) => item.taskType === 'raw-text-chunk-sum-embed' && item.exitCode === 0) ? (
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
