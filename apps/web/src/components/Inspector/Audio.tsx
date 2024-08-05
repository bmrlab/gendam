import { FilePath } from '@/lib/bindings'
import { formatBytes, formatDateTime, formatDuration } from '@/lib/utils'
import { DetailTasks } from '.'
import { PickAssetObject } from '../FileThumb'
import { useSortedTasks } from './hooks'
import Audio from '../FileView/Audio'

export default function AudioDetail({
  data,
  filePath,
}: {
  data: PickAssetObject<'audio'>
  filePath: Omit<FilePath, 'assetObject'>
}) {
  const { sortedTasks } = useSortedTasks(data)

  return (
    <div className="p-3">
      <div className="w-58 bg-app-overlay/50 relative h-48 overflow-hidden p-4">
        <Audio hash={data.hash} duration={data.mediaData.duration} />
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
          <div>{formatBytes(data.size)}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Type</div>
          <div>{data.mimeType}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Duration</div>
          <div>{formatDuration(data.mediaData.duration ?? 0)}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Created</div>
          <div>{formatDateTime(data.createdAt)}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Modified</div>
          <div>{formatDateTime(data.updatedAt)}</div>
        </div>
      </div>
      <div className="bg-app-line mb-3 mt-3 h-px"></div>
      <div className="text-xs">
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Content Hash</div>
          <div>{data.hash}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Asset Object ID</div>
          <div>{data.id}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Transcript Search</div>
          {sortedTasks.some((item) => item.taskType === 'transcript-embedding' && item.exitCode === 0) ? (
            <div className="rounded-full bg-green-100 px-2 text-xs text-green-600">Ready</div>
          ) : (
            <div className="rounded-full bg-orange-100 px-2 text-xs text-orange-600">Not ready</div>
          )}
        </div>
      </div>
      <div className="bg-app-line mb-3 mt-3 h-px"></div>
      <DetailTasks data={data} />
      {/* blank area at the bottom */}
      <div className="mt-6"></div>
    </div>
  )
}
