import ImageViewer from '@/components/FileView/Image'
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
import { rspc } from '@/lib/rspc'

export default function ImageDetail({ filePath, assetObject }: ExtractExplorerItem<'FilePath', 'image'>) {
  const { sortedTasks, handleJobsCancel } = useSortedTasks(assetObject)
  const { data } = rspc.useQuery(['assets.artifacts.image.description', { hash: assetObject.hash }])

  return (
    <InspectorItemContainer>
      <InspectorItemViewer>
        <ImageViewer hash={assetObject.hash} mimeType={assetObject.mimeType} />
      </InspectorItemViewer>

      <InspectorItemFilePath filePath={filePath} />

      <InspectorItemDivider />

      <InspectorItemMetadata data={assetObject}>
        {(assetObject) => (
          <>
            <InspectorItemMetadataItem name="Width">{assetObject.mediaData?.width ?? 0}</InspectorItemMetadataItem>
            <InspectorItemMetadataItem name="Height">{assetObject.mediaData?.width ?? 0}</InspectorItemMetadataItem>
            <InspectorItemMetadataItem name="Color">{assetObject.mediaData?.color}</InspectorItemMetadataItem>
          </>
        )}
      </InspectorItemMetadata>

      <InspectorItemDivider />

      <div className="flex flex-col">
        <div className="text-xs font-semibold mb-2">Description</div>
        <div className="h-48 overflow-y-scroll text-xs select-text cursor-text whitespace-pre-line">{data}</div>
      </div>

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
          {sortedTasks.some((item) => item.taskType === 'image-desc-embed' && item.exitCode === 0) ? (
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
