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
    <>
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
    </>
  )
}

export default CompletedQueueList
