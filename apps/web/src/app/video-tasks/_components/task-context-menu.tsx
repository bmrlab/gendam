import useTaskAction from '@/app/video-tasks/_components/useTaskAction'
import Icon from '@/components/Icon'
import type { VideoWithTasksResult } from '@/lib/bindings'
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuRoot,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@muse/ui/v1/context-menu'
import { PropsWithChildren, ReactNode, useEffect, useMemo } from 'react'
import { useBoundStore } from '../_store'

export type TaskContextMenuProps = PropsWithChildren<{
  fileHash: string
  isNotDone: boolean
  video: VideoWithTasksResult
}>

export default function TaskContextMenu({ video, fileHash, isNotDone, children }: TaskContextMenuProps) {
  const taskListRefetch = useBoundStore.use.taskListRefetch()

  const { handleRegenerate, handleExport, handleCancel } = useTaskAction({ fileHash, video })

  /**
   * 有进行中的任务，定时刷新
   * TODO: 不应该写在这里，要写在列表页上，这里会导致多个定时器，最后请求太多
   */
  // useEffect(() => {
  //   if (isNotDone) {
  //     const timer = setInterval(() => {
  //       taskListRefetch()
  //     }, 10000)
  //     return () => {
  //       clearInterval(timer)
  //     }
  //   }
  // }, [isNotDone, taskListRefetch])

  const options = useMemo<Array<'Separator' | { label: string; icon: ReactNode; handleClick: () => void }>>(() => {
    const processingItem = isNotDone
      ? [
          {
            label: '取消任务',
            icon: <Icon.cancel />,
            handleClick: () => handleCancel(),
          },
        ]
      : []

    return [
      {
        label: '重新触发任务',
        icon: <Icon.regenerate />,
        handleClick: () => handleRegenerate(),
      },
      ...processingItem,
      {
        label: '导出语音转译',
        icon: <Icon.download />,
        handleClick: () => handleExport(),
      },
      'Separator',
      {
        label: '删除任务',
        icon: <Icon.trash />,
        handleClick: () => console.log('删除任务'),
      },
    ]
  }, [handleCancel, handleExport, handleRegenerate, isNotDone])

  return (
    <ContextMenuRoot>
      <ContextMenuTrigger className="flex cursor-default items-center justify-center rounded-md text-sm">
        {children}
      </ContextMenuTrigger>
      <ContextMenuContent className="w-60 rounded-md text-ink bg-app-box border border-app-line p-1 shadow-lg">
        {options.map((o, index) =>
          o === 'Separator' ? (
            <ContextMenuSeparator key={index} className="mx-2.5 bg-app-line" />
          ) : (
            <ContextMenuItem
              key={index}
              inset
              className="relative cursor-default select-none outline-none
              focus:bg-accent focus:text-white hover:bg-accent hover:text-white
              data-[disabled]:pointer-events-none data-[disabled]:opacity-50
              flex items-center justify-start rounded-md px-2 py-2 text-sm"
              onClick={o.handleClick}
            >
              {o.icon}
              <span>{o.label}</span>
            </ContextMenuItem>
          ),
        )}
      </ContextMenuContent>
    </ContextMenuRoot>
  )
}
