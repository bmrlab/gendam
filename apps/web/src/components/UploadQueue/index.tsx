'use client'
import { ExplorerItem } from '@/Explorer/types'
import { useUploadQueueStore, type FileItem } from '@/components/UploadQueue/store'
import { queryClient, rspc } from '@/lib/rspc'
import { Video_File } from '@gendam/assets/images'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import classNames from 'classnames'
import Image from 'next/image'
import { useRouter } from 'next/navigation'
import { HTMLAttributes, PropsWithChildren, useCallback, useEffect, useMemo } from 'react'
// import { twx } from '@/lib/utils'
// const QueueItem = twx.div`flex items-center justify-start pl-2 pr-4 py-2`

const QueueItem = ({
  file,
  children,
  icon,
  status,
  className,
  ...props
}: PropsWithChildren<{
  file: FileItem
  icon?: React.ReactNode
  status?: React.ReactNode
}> &
  HTMLAttributes<HTMLDivElement>) => {
  const splits = file.localFullPath.split('/')
  const fileName = splits.length > 0 ? splits[splits.length - 1] : file.localFullPath
  return (
    <div
      {...props}
      className={classNames(
        'border-app-line group flex items-center justify-start gap-1 border-b px-3 py-2',
        className,
      )}
    >
      <Image src={Video_File} alt="document" className="h-6 w-6" priority></Image>
      <div className="mx-1 flex-1 overflow-hidden text-xs">
        <div className="mb-1 truncate">{fileName}</div>
        {status}
      </div>
      {icon}
      {/* <div className="ml-auto">{children}</div> */}
    </div>
  )
}

const QueueList = () => {
  const uploadQueueStore = useUploadQueueStore()
  const router = useRouter()
  const reveal = useCallback(
    (data: ExplorerItem) => {
      router.push(`/explorer?dir=${data.materializedPath}&id=${data.id}`)
    },
    [router],
  )
  const completedAssetObjectIds = useMemo(() => {
    // assetObject won't be null
    return uploadQueueStore.completed.map((file) => file.assetObject?.id!)
  }, [uploadQueueStore.completed])
  const { data: tasks } = rspc.useQuery(
    [
      'tasks.list',
      {
        filter: { assetObjectIds: completedAssetObjectIds },
      },
    ],
    {
      refetchInterval: 5000,
      enabled: completedAssetObjectIds.length > 0,
    },
  )
  const completedItems = useMemo<
    {
      file: (typeof uploadQueueStore.completed)[number]
      processing: boolean
    }[]
  >(() => {
    return uploadQueueStore.completed.map((file) => {
      const processing = (tasks || []).some(
        (task) => task.assetObjectId === file.assetObject?.id && task.exitCode === null,
      )
      return {
        file,
        processing,
      }
    })
  }, [tasks, uploadQueueStore.completed])

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
      {completedItems.map((item, index) => (
        <QueueItem
          key={index}
          file={item.file}
          // icon={<Icon.Check className="size-4 text-green-600"></Icon.Check>}
          icon={<></>}
          status={
            item.processing ? (
              <div className="text-ink/50 flex items-center gap-2">
                <div>being processed for search</div>
                <Icon.FlashStroke className="h-3 w-3 text-orange-400" />
              </div>
            ) : (
              <div className="text-ink/50 flex items-center gap-2">
                <div>ready for search</div>
                {/* <Icon.Check className="h-3 w-3 text-green-400" /> */}
              </div>
            )
          }
          onClick={() => reveal(item.file)}
        />
      ))}
    </div>
  )
}

const QueueStatus = function () {
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
        <QueueStatus />
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
