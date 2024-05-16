'use client'
import { useUploadQueueStore } from '@/components/UploadQueue/store'
import Icon from '@gendam/ui/icons'
import { useMemo } from 'react'

const QueueStatusHeader = function () {
  const uploadQueueStore = useUploadQueueStore()

  const counts = useMemo(() => {
    const pending = uploadQueueStore.queue.length + (uploadQueueStore.uploading ? 1 : 0)
    const success = uploadQueueStore.completed.length
    const failed = uploadQueueStore.failed.length
    const total = pending + success + failed
    return { pending, success, failed, total }
  }, [uploadQueueStore])

  if (counts.pending > 0) {
    return (
      <>
        <div>
          <Icon.Loading className="size-5 animate-spin text-orange-600"></Icon.Loading>
        </div>
        <div className="flex-1">
          <span className="text-xs font-medium">Importing</span>
          <span className="text-ink/50 ml-2 text-xs">
            {counts.pending} of {counts.total} files left
          </span>
        </div>
      </>
    )
  } else if (counts.failed > 0) {
    return (
      <>
        <div className="size-4 rounded-full bg-red-500 p-[2px] text-white">
          <Icon.Close className="size-full"></Icon.Close>
        </div>
        <div className="flex-1">
          <span className="text-xs font-medium">Import failed</span>
          <span className="text-ink/50 ml-2 text-xs">
            {counts.failed} of {counts.total} files failed
          </span>
        </div>
      </>
    )
  } else if (counts.success > 0) {
    return (
      <>
        <div className="size-4 rounded-full bg-green-500 p-[2px] text-white">
          <Icon.Check className="size-full"></Icon.Check>
        </div>
        <div className="flex-1">
          <span className="text-xs font-medium">Import completed</span>
          <span className="text-ink/50 ml-2 text-xs">{counts.success} files imported</span>
        </div>
      </>
    )
  } else {
    return (
      <>
        <div className="flex-1">
          <span className="text-xs font-medium">No files to import</span>
        </div>
      </>
    )
  }
}

export default QueueStatusHeader
