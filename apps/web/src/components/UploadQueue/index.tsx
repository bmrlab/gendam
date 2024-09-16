import { useUploadQueueStore } from '@/components/UploadQueue/store'
import { queryClient, rspc } from '@/lib/rspc'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import { useEffect } from 'react'
import CompletedQueueList from './CompletedQueueList'
import QueueItem from './QueueItem'
import QueueStatusHeader from './QueueStatusHeader'
// import { twx } from '@/lib/utils'
// const QueueItem = twx.div`flex items-center justify-start pl-2 pr-4 py-2`

const QueueList = () => {
  const uploadQueueStore = useUploadQueueStore()
  return (
    <div className="h-80 w-80 overflow-y-auto overflow-x-hidden">
      {uploadQueueStore.uploading ? (
        <QueueItem
          file={uploadQueueStore.uploading}
          icon={<Icon.Loading className="size-4 animate-spin text-orange-600"></Icon.Loading>}
          status={<div className="text-ink/50">Uploading</div>}
        />
      ) : null}
      {uploadQueueStore.queue.map((file, index) => (
        <QueueItem key={index} file={file} status={<div className="text-ink/50">Wait for import</div>} />
      ))}
      {uploadQueueStore.failed.map((file, index) => (
        <QueueItem
          key={index}
          file={file}
          icon={
            <>
              <Icon.Close className="size-4 text-red-600 group-hover:hidden"></Icon.Close>
              <Button
                variant="ghost"
                size="xs"
                className="hidden text-xs text-orange-600 group-hover:block"
                onClick={() => uploadQueueStore.retryFailed(file)}
              >
                Retry
              </Button>
            </>
          }
          status={<div className="text-ink/50">Failed</div>}
        />
      ))}
      <CompletedQueueList />
    </div>
  )
}

export default function UploadQueue({ close }: { close: () => void }) {
  const uploadQueueStore = useUploadQueueStore()
  const uploadMut = rspc.useMutation(['assets.create_asset_object'])

  useEffect(() => {
    // useUploadQueueStore.subscribe((e) => {})
    const uploading = uploadQueueStore.nextUploading()
    if (uploading) {
      const { materializedPath, name, localFullPath } = uploading
      uploadMut
        .mutateAsync({ materializedPath, name, localFullPath })
        .then((filePathData) => {
          uploadQueueStore.completeUploading(filePathData)
        })
        .catch(() => {
          uploadQueueStore.failedUploading()
        })
        .finally(() => {
          queryClient.invalidateQueries({
            queryKey: ['assets.list', { materializedPath: materializedPath }],
          })
        })
    }
  }, [uploadQueueStore, uploadMut])

  return (
    <>
      <div className="border-app-line flex h-12 items-center justify-between gap-2 border-b pl-4 pr-2">
        <QueueStatusHeader />
        <div onClick={() => uploadQueueStore.clear()} className="hover:bg-app-hover h-5 w-5 rounded p-1">
          <Icon.Trash className="h-full w-full" />
        </div>
        <div onClick={() => close()} className="hover:bg-app-hover h-5 w-5 rounded p-1">
          <Icon.Close className="h-full w-full" />
        </div>
      </div>
      <QueueList />
    </>
  )
}
