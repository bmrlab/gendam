import { useBoundStore } from '@/app/video-tasks/_store'
import { VideoWithTasksResult } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import Icon from '@gendam/ui/icons'
import type { ReactNode } from 'react'
import { useCallback, useMemo } from 'react'
import { toast } from 'sonner'
import { TaskStatus, getTaskStatus } from './utils'
import { useRouter } from 'next/navigation'

export type TaskActionOption =
  | 'Separator'
  | {
      disabled?: boolean
      variant?: 'accent' | 'destructive'
      label: string
      icon: ReactNode
      handleSelect: () => void
    }

function useTaskAction(videos: VideoWithTasksResult[]) {
  const router = useRouter()
  const taskListRefetch = useBoundStore.use.taskListRefetch()

  const { mutateAsync: regenerateTask } = rspc.useMutation(['video.tasks.regenerate'])
  const { mutateAsync: cancelTask } = rspc.useMutation(['video.tasks.cancel'])

  const isBatchSelected = useMemo(() => videos.length > 1, [videos])

  const handleSingleReveal = useCallback(() => {
    const item = videos[0];
    router.push('/explorer?dir=' + item.materializedPath)
  }, [router, videos])

  const handleSingleRegenerate = useCallback(
    async (param: { path: string; id: number }) => {
      try {
        await regenerateTask({
          assetObjectId: param.id,
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
            onClick: () => handleSingleRegenerate(param),
          },
        })
      }
    },
    [regenerateTask, taskListRefetch],
  )

  const handleBatchRegenerate = useCallback(() => {
    videos.forEach(async (item) => {
      await handleSingleRegenerate({
        path: item.materializedPath,
        id: item.assetObject.id,
      })
    })
  }, [handleSingleRegenerate, videos])

  const handleSingleCancel = useCallback(
    async (id: number) => {
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
      await handleSingleCancel(item.assetObject.id)
    })
  }, [handleSingleCancel, videos])

  return {
    handleRegenerate: handleBatchRegenerate,
    handleCancel: handleBatchCancel,
    handleReveal: handleSingleReveal,
  }
}

export function useTaskActionOptions(videos: VideoWithTasksResult[]) {
  const { handleRegenerate, handleCancel, handleReveal } = useTaskAction(videos)

  const options = useMemo(() => {
    const options: Array<TaskActionOption> = [
      {
        label: 'Re-process job',
        icon: <Icon.Cycle className="size-4" />,
        handleSelect: () => handleRegenerate(),
      },
      {
        disabled: videos.length > 1,
        label: 'Reveal in explorer',
        icon: <Icon.MagnifyingGlass className="size-4" />,
        handleSelect: () => handleReveal(),
      }
    ]
    if (
      videos
        .map((v) => v.tasks)
        .flat()
        .some((task) => [TaskStatus.None, TaskStatus.Processing].includes(getTaskStatus(task)))
    ) {
      /**
       * 未开始, 正在进行的, 可以取消
       * 已完成, 已取消, 出错的, 不可以取消
       */
      options.push({
        label: 'Cancel job',
        icon: <Icon.CloseRounded className="size-4" />,
        handleSelect: () => handleCancel(),
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
  }, [handleCancel, handleRegenerate, handleReveal, videos])

  return {
    options,
  }
}
