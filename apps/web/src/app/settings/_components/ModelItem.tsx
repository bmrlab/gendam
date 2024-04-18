import { AIModelCategory, AIModelResult } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { useCallback, useEffect } from 'react'
import { toast } from 'sonner'

interface ModelItemProps {
  model: AIModelResult
  category: AIModelCategory
  activated: boolean
}

export function ModelItem({ model, activated, category }: ModelItemProps) {
  const { mutateAsync } = rspc.useMutation(['libraries.models.set_model'])
  const { mutateAsync: downloadAsync } = rspc.useMutation(['libraries.models.download_model'])
  const { data: modelResult, refetch } = rspc.useQuery(['libraries.models.get_model', model.info.id], {
    enabled: false,
    initialData: model,
  })

  const handleChangeModel = useCallback(async () => {
    if (activated || !modelResult || !modelResult?.status.downloaded) {
      return
    }
    await mutateAsync({
      category,
      modelId: modelResult.info.id,
    })
    toast.success(`Set ${category} model to ${modelResult.info.id}`)
  }, [activated, category, modelResult, mutateAsync])

  const handleDownload = useCallback(async () => {
    if (!modelResult) return

    if (modelResult.status.downloaded || modelResult.status.downloadStatus) {
      return
    }
    await downloadAsync({
      modelId: modelResult.info.id,
    })
    await refetch()
    toast.success(`Trigger model downloading: ${modelResult.info.id}`)
  }, [modelResult, downloadAsync, refetch])

  useEffect(() => {
    let timer: NodeJS.Timeout | undefined

    if (modelResult?.status.downloadStatus) {
      timer = setInterval(() => {
        refetch()
      }, 500)
    }

    return () => timer && clearInterval(timer)
  }, [modelResult, refetch])

  if (!modelResult) return <></>

  return (
    <div className="flex w-full">
      <div
        className="hover:bg-app-hover cursor-pointer rounded-lg p-2"
        onClick={() => {
          if (modelResult.status.downloaded) handleChangeModel()
          else if (!modelResult.status.downloaded && !modelResult.status.downloadStatus) handleDownload()
        }}
      >
        {modelResult.info.id}: {modelResult.info.title}{' '}
        {activated
          ? '✅'
          : modelResult?.status.downloadStatus
            ? `⏳ ${modelResult?.status.downloadStatus.downloadedBytes}B / ${modelResult?.status.downloadStatus.totalBytes}B`
            : modelResult?.status.downloaded
              ? '🔥'
              : '⏬'}
      </div>
    </div>
  )
}
