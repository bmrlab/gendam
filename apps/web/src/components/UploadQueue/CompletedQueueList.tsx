'use client'
import { ExplorerItem } from '@/Explorer/types'
import { useUploadQueueStore } from '@/components/UploadQueue/store'
import { rspc } from '@/lib/rspc'
import Icon from '@gendam/ui/icons'
import { useRouter } from 'next/navigation'
import { useCallback, useMemo } from 'react'
import QueueItem from './QueueItem'
// import { twx } from '@/lib/utils'
// const QueueItem = twx.div`flex items-center justify-start pl-2 pr-4 py-2`

const CompletedQueueList = () => {
  const uploadQueueStore = useUploadQueueStore()
  const router = useRouter()
  const reveal = useCallback(
    (data: ExplorerItem) => {
      router.push(`/explorer?dir=${data.materializedPath}&id=${data.id}`)
    },
    [router],
  )
  const { data: filePathsInProcess } = rspc.useQuery(['tasks.get_assets_in_process'], {
    refetchInterval: 5000,
  })
  // 合并刚上传的 filePath 和正在处理中的 filePath
  const mergedCompletedItems = useMemo<
    {
      file: ExplorerItem
      processing: boolean
    }[]
  >(() => {
    const completedIds = new Set(uploadQueueStore.completed.map((f) => f.id))
    const processingIds = new Set(filePathsInProcess?.map((f) => f.id))
    const completed = uploadQueueStore.completed.map((f) => ({
      file: f,
      processing: processingIds.has(f.id),
    }))
    const inProcess = (filePathsInProcess || [])
      .filter((f) => !completedIds.has(f.id))
      .map((f) => ({
        file: f,
        processing: true,
      }))
    return [...completed, ...inProcess]
  }, [filePathsInProcess, uploadQueueStore.completed])

  return (
    <>
      {mergedCompletedItems.map((item, index) => (
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
    </>
  )
}

export default CompletedQueueList
