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
      <ContextMenuContent className="muse-border w-[215px] bg-[#F4F5F5] py-2 shadow-md">
        {options.map((o, index) =>
          o === 'Separator' ? (
            <ContextMenuSeparator key={index} className="mx-2.5 bg-[#DDDDDE]" />
          ) : (
            <ContextMenuItem
              key={index}
              inset
              className="flex gap-1.5 px-2.5 py-[3.5px] text-[13px] leading-[18.2px] transition focus:bg-[#017AFF] focus:text-white data-[disabled]:text-[#AAADB2] data-[disabled]:opacity-100"
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
