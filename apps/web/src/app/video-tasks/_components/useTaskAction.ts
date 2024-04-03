import { TaskContextMenuProps } from '@/app/video-tasks/_components/TaskContextMenu'
import { useBoundStore } from '@/app/video-tasks/_store'
import { AudioDialogEnum } from '@/app/video-tasks/_store/audio-dialog'
import { rspc } from '@/lib/rspc'
import { useCallback, useMemo } from 'react'
import { toast } from 'sonner'

export type TaskActionProps = Pick<TaskContextMenuProps, 'fileHash' | 'video'>

export default function useTaskAction({ fileHash, video }: TaskActionProps) {
  const videoSelected = useBoundStore.use.videoSelected()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()
  const setAudioDialogProps = useBoundStore.use.setAudioDialogProps()
  const setAudioDialogOpen = useBoundStore.use.setIsOpenAudioDialog()
  const taskListRefetch = useBoundStore.use.taskListRefetch()

  const { mutateAsync: regenerateTask } = rspc.useMutation(['video.tasks.regenerate'])
  const { mutateAsync: cancelTask } = rspc.useMutation(['video.tasks.cancel'])

  const isBatchSelected = useMemo(() => videoSelected.length > 1, [videoSelected])

  const { assetObjectId, materializedPath } = useMemo(
    () => ({
      assetObjectId: video.assetObject.id,
      materializedPath: video.materializedPath,
    }),
    [video.assetObject.id, video.materializedPath],
  )

  const handleSingleExport = useCallback(() => {
    setAudioDialogProps({
      type: AudioDialogEnum.single,
      title: '导出语音转译',
      params: {
        fileHash,
      },
    })
    setIsOpenAudioDialog(true)
  }, [fileHash, setAudioDialogProps, setIsOpenAudioDialog])

  const handleBatchExport = () => {
    let orderVideoSelected = [...videoSelected]
    orderVideoSelected.sort((a, b) => a.assetObject.id - b.assetObject.id)
    setAudioDialogProps({
      type: AudioDialogEnum.batch,
      title: '批量导出语音转译',
      params: orderVideoSelected.map((item) => ({
        id: item.assetObject.hash, // TODO: 这里回头要改成 assetObjectId, 但是对 audio export 功能改动较大
        label: item.name,
        assetObjectId: item.assetObject.id,
        assetObjectHash: item.assetObject.hash,
      })),
    })
    setAudioDialogOpen(true)
  }

  const handleRegenerate = useCallback(
    async (param?: { path: string; id: number }) => {
      try {
        await regenerateTask({
          assetObjectId: param?.id ?? assetObjectId,
        })
        await taskListRefetch()
        toast.success('重新触发任务成功', {
          action: {
            label: 'Dismiss',
            onClick: () => {},
          },
        })
      } catch (e) {
        console.error(e)
        toast.error('重新触发任务失败', {
          action: {
            label: 'Retry',
            onClick: () => handleRegenerate(param),
          },
        })
      }
    },
    [assetObjectId, regenerateTask, taskListRefetch],
  )

  const handleBatchRegenerate = useCallback(() => {
    videoSelected.forEach(async (item) => {
      await handleRegenerate({
        path: item.materializedPath,
        id: item.assetObject.id,
      })
    })
  }, [handleRegenerate, videoSelected])

  const handleCancel = useCallback(
    async (id?: number) => {
      await cancelTask({
        assetObjectId: id ?? assetObjectId,
      })
      await taskListRefetch()
      toast.success('取消任务成功', {
        action: {
          label: 'Dismiss',
          onClick: () => {},
        },
      })
    },
    [assetObjectId, cancelTask, taskListRefetch],
  )

  const handleBatchCancel = useCallback(() => {
    videoSelected.forEach(async (item) => {
      await handleCancel(item.assetObject.id)
    })
  }, [handleCancel, videoSelected])

  return {
    handleExport: isBatchSelected ? handleBatchExport : handleSingleExport,
    handleRegenerate: isBatchSelected ? handleBatchRegenerate : handleRegenerate,
    handleCancel: isBatchSelected ? handleBatchCancel : handleCancel,
  }
}
