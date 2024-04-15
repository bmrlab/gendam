import { useBoundStore } from '@/app/video-tasks/_store'
import { AudioDialogEnum } from '@/app/video-tasks/_store/audio-dialog'
import { VideoWithTasksResult } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import Icon from '@muse/ui/icons'
import type { ReactNode } from 'react'
import { useCallback, useMemo } from 'react'
import { toast } from 'sonner'
import { TaskStatus, getTaskStatus, isNotDone } from './utils'

export type TaskActionOption =
  | 'Separator'
  | {
      disabled?: boolean
      variant?: 'accent' | 'destructive'
      label: string
      icon: ReactNode
      handleClick: () => void
    }

function useTaskAction(videos: VideoWithTasksResult[]) {
  // // 右键的时候任务自然被选中了，直接从所有选中的任务中获取数据即可
  // const videos = useBoundStore.use.videos()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()
  const setAudioDialogProps = useBoundStore.use.setAudioDialogProps()
  const setAudioDialogOpen = useBoundStore.use.setIsOpenAudioDialog()
  const taskListRefetch = useBoundStore.use.taskListRefetch()

  const { mutateAsync: regenerateTask } = rspc.useMutation(['video.tasks.regenerate'])
  const { mutateAsync: cancelTask } = rspc.useMutation(['video.tasks.cancel'])

  const handleSingleExport = useCallback(() => {
    setAudioDialogProps({
      type: AudioDialogEnum.single,
      title: 'Export Transcript',
      params: {
        fileHash: videos.at(0)?.assetObject.hash!,
      },
    })
    setIsOpenAudioDialog(true)
  }, [setAudioDialogProps, setIsOpenAudioDialog, videos])

  const handleBatchExport = () => {
    let ordervideos = [
      ...videos.filter((v) => v.tasks.some((t) => t.taskType === 'Transcript' && getTaskStatus(t) === TaskStatus.Done)),
    ]
    ordervideos.sort((a, b) => a.assetObject.id - b.assetObject.id)
    setAudioDialogProps({
      type: AudioDialogEnum.batch,
      title: 'Bulk Transcript Export',
      params: ordervideos.map((item) => ({
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
      if (!param?.id) return

      try {
        await regenerateTask({
          assetObjectId: param.id,
          preserveArtifacts: false,
        })
        await taskListRefetch()
        toast.success('Successfully re-process job', {
          action: {
            label: 'Dismiss',
            onClick: () => {},
          },
        })
      } catch (e) {
        console.error(e)
        toast.error('Failed re-process job', {
          action: {
            label: 'Retry',
            onClick: () => handleRegenerate(param),
          },
        })
      }
    },
    [regenerateTask, taskListRefetch],
  )

  const handleBatchRegenerate = useCallback(() => {
    videos.forEach(async (item) => {
      await handleRegenerate({
        path: item.materializedPath,
        id: item.assetObject.id,
      })
    })
  }, [handleRegenerate, videos])

  const handleCancel = useCallback(
    async (id?: number) => {
      if (!id) return

      await cancelTask({
        assetObjectId: id,
        taskTypes: null,
      })
      await taskListRefetch()
      toast.success('Job cancelled', {
        action: {
          label: 'Dismiss',
          onClick: () => {},
        },
      })
    },
    [cancelTask, taskListRefetch],
  )

  const handleBatchCancel = useCallback(() => {
    videos.forEach(async (item) => {
      await handleCancel(item.assetObject.id)
    })
  }, [handleCancel, videos])

  return {
    handleExport: handleBatchExport,
    handleRegenerate: handleBatchRegenerate,
    handleCancel: handleBatchCancel,
  }
}

export function useTaskActionOptions(videos: VideoWithTasksResult[]) {
  const { handleExport, handleRegenerate, handleCancel } = useTaskAction(videos)

  const options = useMemo(() => {
    const options: Array<TaskActionOption> = [
      {
        label: 'Re-process job ',
        icon: <Icon.Cycle className="size-4" />,
        handleClick: () => handleRegenerate(),
      },
    ]

    if (isNotDone(videos.map((v) => v.tasks).flat())) {
      options.push({
        label: 'Cancel job',
        icon: <Icon.CloseRounded className="size-4" />,
        handleClick: () => handleCancel(),
      })
    }

    if (
      !!videos.find((v) => v.tasks.some((t) => t.taskType === 'Transcript' && getTaskStatus(t) === TaskStatus.Done))
    ) {
      options.push({
        label: 'Export transcript',
        icon: <Icon.Download className="size-4" />,
        handleClick: () => handleExport(),
      })
    }

    return [
      ...options,
      'Separator',
      {
        disabled: true,
        variant: 'destructive',
        label: 'Delete job',
        icon: <Icon.Trash className="size-4" />,
        handleClick: () => console.log('Delete job'),
      },
    ] as Array<TaskActionOption>
  }, [handleCancel, handleExport, handleRegenerate, videos])

  return {
    options,
  }
}
