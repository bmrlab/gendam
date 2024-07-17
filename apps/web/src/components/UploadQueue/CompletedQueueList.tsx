'use client'
import { type FilePath } from '@/lib/bindings'
import { useUploadQueueStore } from '@/components/UploadQueue/store'
import Icon from '@gendam/ui/icons'
import { useRouter } from 'next/navigation'
import { useCallback, useMemo } from 'react'
import QueueItem from './QueueItem'
import { useTranslation } from 'react-i18next'

const CompletedQueueList = () => {
  const { t } = useTranslation()
  const uploadQueueStore = useUploadQueueStore()
  const router = useRouter()
  const reveal = useCallback(
    (data: FilePath) => {
      router.push(`/explorer?dir=${data.materializedPath}&id=${data.id}`)
    },
    [router],
  )

  // 合并刚上传的 filePath 和正在处理中的 filePath
  const mergedCompletedItems = useMemo<
    {
      file: FilePath
      processing: boolean
    }[]
  >(() => {
    const completedIds = new Set(uploadQueueStore.completed.map((f) => f.id))
    const processingIds = new Set(uploadQueueStore.inProcess.map((f) => f.id))
    const completed = uploadQueueStore.completed.map((f) => ({
      file: f,
      processing: processingIds.has(f.id),
    }))
    const inProcess = uploadQueueStore.inProcess
      .filter((f) => !completedIds.has(f.id))
      .map((f) => ({
        file: f,
        processing: true,
      }))
    return [...completed, ...inProcess]
  }, [uploadQueueStore.inProcess, uploadQueueStore.completed])

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
                <div>{t('uploadQueue.processing')}</div>
                <Icon.FlashStroke className="h-3 w-3 text-orange-400" />
              </div>
            ) : (
              <div className="text-ink/50 flex items-center gap-2">
                <div>{t('uploadQueue.processed')}</div>
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
