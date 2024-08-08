import { ExtractExplorerItem } from '@/Explorer/types'
import { formatBytes, formatDateTime } from '@/lib/utils'
import { Video } from '../../FileView/Video'
import { DetailTasks } from '../../Inspector'
import { useSortedTasks } from '../../Inspector/hooks'
import RawTextViewer from '@/components/FileView/RawText'

export default function RawTextDetail({ filePath, assetObject }: ExtractExplorerItem<'FilePath', 'rawText'>) {
  const { sortedTasks } = useSortedTasks(assetObject)

  return (
    <div className="p-3">
      <div className="w-58 bg-app-overlay/50 relative h-48 overflow-hidden">
        <RawTextViewer hash={assetObject.hash} />
      </div>

      <div className="mt-3 overflow-hidden">
        <div className="text-ink line-clamp-2 break-all text-sm font-medium">{filePath.name}</div>
        <div className="text-ink/50 mt-1 line-clamp-2 text-xs">Location {filePath.materializedPath}</div>
      </div>

      <div className="bg-app-line mb-3 mt-3 h-px"></div>
      <div className="text-xs">
        <div className="text-md font-medium">Information</div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Size</div>
          <div>{formatBytes(assetObject.size)}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Type</div>
          <div>{assetObject.mimeType}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Text Count</div>
          <div>{assetObject.mediaData?.text_count ?? '-'}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Created</div>
          <div>{formatDateTime(assetObject.createdAt)}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Modified</div>
          <div>{formatDateTime(assetObject.updatedAt)}</div>
        </div>
      </div>
      <div className="bg-app-line mb-3 mt-3 h-px"></div>
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
      <div className="bg-app-line mb-3 mt-3 h-px"></div>
      <DetailTasks data={assetObject} />
      {/* blank area at the bottom */}
      <div className="mt-6"></div>
    </div>
  )
}
