import { useUploadQueueStore } from '@/components/UploadQueue/store'
import { queryClient, rspc } from '@/lib/rspc'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import { useCallback, useEffect } from 'react'
import CompletedQueueList from './CompletedQueueList'
import QueueItem from './QueueItem'
import QueueStatusHeader from './QueueStatusHeader'
// import { twx } from '@/lib/utils'
// const QueueItem = twx.div`flex items-center justify-start pl-2 pr-4 py-2`

const uploadChunk = async (
  fileName: string,
  chunk: ArrayBuffer,
  chunkSize: number,
  chunkIndex: number,
  totalChunks: number,
) => {
  const url = `http://localhost:3001/_storage/localhost/upload_file_chunk_to_temp/`
  const chunkData = {
    fileName,
    chunkSize,
    chunkIndex,
    totalChunks,
    chunk: Array.from(new Uint8Array(chunk)),
  }
  try {
    const response = await fetch(url, {
      method: 'POST',
      body: JSON.stringify(chunkData),
      headers: {
        'Content-Type': 'application/json',
      },
    })

    if (!response.ok) {
      throw new Error(`Upload failed: ${response.statusText}`)
    }

    return await response.json()
  } catch (error) {
    console.error('Upload failed:', error)
    throw error
  }
}

const uploadFile = async (fileName: string, file: File) => {
  // 这里一定要 async 上传，不能 await 然后 block for loop，因为这样会消耗更多内存导致最终浏览器崩溃
  {
    const uuid = crypto.randomUUID()
    const lastDotIndex = fileName.lastIndexOf('.')
    fileName =
      lastDotIndex !== -1
        ? `${fileName.slice(0, lastDotIndex)}_${uuid}${fileName.slice(lastDotIndex)}`
        : `${fileName}_${uuid}`
  }
  const parallelUploads = 10
  const chunkSize = 1 * 1024 * 1024 // 1MB chunks
  const totalChunks = Math.ceil(file.size / chunkSize)
  const promises = []
  let fullPath = ''
  const promise = async (i: number, start: number, end: number) => {
    const chunk = await file.slice(start, end).arrayBuffer()
    const res = await uploadChunk(fileName, chunk, chunkSize, i, totalChunks)
    fullPath = res.fullPath
    console.log('File chunk uploaded', res)
  }
  for (let i = 0; i < totalChunks; i++) {
    const start = i * chunkSize
    const end = Math.min(start + chunkSize, file.size)
    promises.push(promise(i, start, end))
    if (i === 0 || promises.length % parallelUploads === 0) {
      // 第 0 个 chunk 要单独来，确保先创建文件
      await Promise.all(promises)
      promises.length = 0
    }
  }
  if (promises.length > 0) {
    await Promise.all(promises)
    promises.length = 0
  }
  return fullPath
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
  const createAssetObjectMut = rspc.useMutation(['assets.create_asset_object'])

  const completeUploading = uploadQueueStore.completeUploading
  const failedUploading = uploadQueueStore.failedUploading
  const createAssetObject = useCallback(
    async (materializedPath: string, name: string, localFullPath: string) => {
      return createAssetObjectMut
        .mutateAsync({ materializedPath, name, localFullPath })
        .then((filePathData) => {
          completeUploading(filePathData)
        })
        .catch(() => {
          failedUploading()
        })
        .finally(() => {
          queryClient.invalidateQueries({
            queryKey: ['assets.list', { materializedPath: materializedPath }],
          })
        })
    },
    [createAssetObjectMut, completeUploading, failedUploading],
  )

  useEffect(() => {
    // useUploadQueueStore.subscribe((e) => {})
    const uploading = uploadQueueStore.nextUploading()
    if (uploading) {
      const { materializedPath, name, dataType, payload } = uploading
      if (dataType === 'path') {
        createAssetObject(materializedPath, name, payload)
      } else if (dataType === 'file') {
        // const name = file.name
        uploadFile(name, payload)
          .then((fullPath) => createAssetObject(materializedPath, name, fullPath))
          .catch((e) => console.error('File upload failed', e))
      }
    }
  }, [uploadQueueStore, createAssetObject])

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
