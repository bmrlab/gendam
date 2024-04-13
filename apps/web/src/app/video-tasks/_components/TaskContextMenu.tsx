import useTaskAction from '@/app/video-tasks/_components/useTaskAction'
import type { VideoWithTasksResult } from '@/lib/bindings'
import Icon from '@muse/ui/icons'
import { ContextMenu } from '@muse/ui/v2/context-menu'
import { PropsWithChildren, ReactNode, useMemo } from 'react'
// import { useBoundStore } from '../_store'

export type TaskContextMenuProps = PropsWithChildren<{
  fileHash: string
  isNotDone: boolean
  video: VideoWithTasksResult
}>

export default function TaskContextMenu({ video, fileHash, isNotDone, children }: TaskContextMenuProps) {
  const { handleRegenerate, handleExport, handleCancel } = useTaskAction({ fileHash, video })

  const options = useMemo<
    Array<
      | 'Separator'
      | {
          disabled?: boolean
          variant?: 'accent' | 'destructive'
          label: string
          icon: ReactNode
          handleClick: () => void
        }
    >
  >(() => {
    const processingItem = isNotDone
      ? [
          {
            label: 'Cancel job',
            icon: <Icon.CloseRounded className="size-4" />,
            handleClick: () => handleCancel(),
          },
        ]
      : []

    return [
      {
        label: 'Re-process job ',
        icon: <Icon.Cycle className="size-4" />,
        handleClick: () => handleRegenerate(),
      },
      ...processingItem,
      {
        label: 'Export transcript',
        icon: <Icon.Download className="size-4" />,
        handleClick: () => handleExport(),
      },
      'Separator',
      {
        disabled: true,
        variant: 'destructive',
        label: 'Delete job',
        icon: <Icon.Trash className="size-4" />,
        handleClick: () => console.log('Delete job'),
      },
    ]
  }, [handleCancel, handleExport, handleRegenerate, isNotDone])

  return (
    <ContextMenu.Root>
      <ContextMenu.Trigger>{children}</ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content>
          {options.map((o, index) =>
            o === 'Separator' ? (
              <ContextMenu.Separator key={index} />
            ) : (
              <ContextMenu.Item key={index} onClick={o.handleClick} variant={o.variant} disabled={o.disabled}>
                {o.icon}
                <span>{o.label}</span>
              </ContextMenu.Item>
            ),
          )}
        </ContextMenu.Content>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  )
}
