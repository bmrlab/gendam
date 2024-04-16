'use client'
import Icon from '@muse/ui/icons'
import { rspc, queryClient } from '@/lib/rspc'
import { FileItem, useUploadQueueStore } from '@/store/uploadQueue'
import { Video_File } from '@muse/assets/images'
import classNames from 'classnames'
import Image from 'next/image'
import { PropsWithChildren, useCallback, useEffect, useMemo, useState } from 'react'
import { Button } from '@muse/ui/v2/button'
// import { twx } from '@/lib/utils'
// const QueueItem = twx.div`flex items-center justify-start pl-2 pr-4 py-2`

const QueueItem = ({ file, children, icon, status }: PropsWithChildren<{
  file: FileItem
  icon?: React.ReactNode
  status?: React.ReactNode
}>) => {
  const splits = file.localFullPath.split('/')
  const fileName = splits.length > 0 ? splits[splits.length - 1] : file.localFullPath
  return (
    <div className="group flex items-center justify-start gap-1 py-2 px-3 border-b border-app-line">
      <Image src={Video_File} alt="document" className="w-6 h-6" priority></Image>
      <div className="flex-1 mx-1 text-xs overflow-hidden">
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
        <QueueItem
          key={index} file={file}
          status={<div className="text-ink/50">Wait for upload</div>}
        />
      ))}
      {uploadQueueStore.failed.map((file, index) => (
        <QueueItem
          key={index} file={file}
          icon={<>
            <Icon.Close className="group-hover:hidden size-4 text-red-600"></Icon.Close>
            <Button
              variant="ghost" size="xs"
              className="hidden group-hover:block text-xs text-orange-600"
              onClick={() => uploadQueueStore.retryFailed(file)}
            >Retry</Button>
          </>}
          status={<div className="text-ink/50">Failed</div>}
        />
      ))}
      {uploadQueueStore.completed.map((file, index) => (
        <QueueItem
          key={index} file={file}
          icon={<Icon.Check className="size-4 text-green-600"></Icon.Check>}
          status={<div className="text-ink/50">Uploaded</div>}
        />
      ))}
    </div>
  )
}

const QueueStatus = function() {
  const uploadQueueStore = useUploadQueueStore()

  const counts = useMemo(() => {
    const pending = uploadQueueStore.queue.length + (uploadQueueStore.uploading ? 1 : 0)
    const success = uploadQueueStore.completed.length
    const failed = uploadQueueStore.failed.length
    const total = pending + success + failed
    return { pending, success, failed, total }
  }, [uploadQueueStore])

  if (counts.pending > 0) {
    return <>
      <div>
        <Icon.Loading className="size-5 animate-spin text-orange-600"></Icon.Loading>
      </div>
      <div className="flex-1">
        <span className="text-xs font-medium">Uploading</span>
        <span className="text-xs text-ink/50 ml-2">
          {counts.pending} of {counts.total} files left
        </span>
      </div>
    </>
  } else if (counts.failed > 0) {
    return <>
      <div className="size-4 p-[2px] bg-red-500 text-white rounded-full">
        <Icon.Close className="size-full"></Icon.Close>
      </div>
      <div className="flex-1">
        <span className="text-xs font-medium">Upload failed</span>
        <span className="text-xs text-ink/50 ml-2">
          {counts.failed} of {counts.total} files failed
        </span>
      </div>
    </>
  } else if (counts.success > 0){
    return <>
      <div className="size-4 p-[2px] bg-green-500 text-white rounded-full">
        <Icon.Check className="size-full"></Icon.Check>
      </div>
      <div className="flex-1">
        <span className="text-xs font-medium">Upload completed</span>
        <span className="text-xs text-ink/50 ml-2">
          {counts.success} files uploaded
        </span>
      </div>
    </>
  } else {
    return <>
      <div className="flex-1">
        <span className="text-xs font-medium">No files to upload</span>
      </div>
    </>
  }
}

export default function UploadQueue({ close }: {
  close: () => void
}) {
  const uploadQueueStore = useUploadQueueStore()
  const uploadMut = rspc.useMutation(['assets.create_asset_object'])

  useEffect(() => {
    // useUploadQueueStore.subscribe((e) => {})
    const uploading = uploadQueueStore.nextUploading()
    if (uploading) {
      const { materializedPath, name, localFullPath } = uploading
      uploadMut.mutateAsync({ materializedPath, name, localFullPath }).then(() => {
        uploadQueueStore.completeUploading()
      }).catch(() => {
        uploadQueueStore.failedUploading()
      }).finally(() => {
        queryClient.invalidateQueries({
          queryKey: ['assets.list', { materializedPath: materializedPath }]
        })
      })
    }
  }, [uploadQueueStore, uploadMut])

  return (
    <div className={classNames(
      "absolute z-10 bottom-9 left-0 overflow-hidden",
      "rounded-md text-ink bg-app-box border border-app-line shadow-lg",
    )}>
      <div className="flex items-center justify-between gap-2 pl-4 pr-2 h-12 border-b border-app-line">
        <QueueStatus />
        <div onClick={() => uploadQueueStore.clear()} className="h-5 w-5 p-1 rounded hover:bg-app-hover">
          <Icon.Trash className="w-full h-full" />
        </div>
        <div onClick={() => close()} className="h-5 w-5 p-1 rounded hover:bg-app-hover">
          <Icon.Close className="w-full h-full" />
        </div>
      </div>
      <QueueList />
    </div>
  )
}
