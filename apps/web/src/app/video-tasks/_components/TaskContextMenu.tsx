import { useTaskActionOptions } from '@/app/video-tasks/_components/useTaskActionOptions'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { PropsWithChildren } from 'react'
import { useBoundStore } from '../_store'

export type TaskContextMenuProps = PropsWithChildren

export default function TaskContextMenu({ children }: TaskContextMenuProps) {
  const videoSelected = useBoundStore.use.videoSelected()

  const { options } = useTaskActionOptions(videoSelected)

  return (
    <ContextMenu.Root>
      <ContextMenu.Trigger>{children}</ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content onClick={(e) => e.stopPropagation()}>
          {options.map((o, index) =>
            o === 'Separator' ? (
              <ContextMenu.Separator key={index} />
            ) : (
              <ContextMenu.Item key={index} onSelect={o.handleSelect} variant={o.variant} disabled={o.disabled}>
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
