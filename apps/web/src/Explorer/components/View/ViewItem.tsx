'use client'
import { useExplorerContext, useExplorerViewContext } from '@/Explorer/hooks'
import { useExplorerStore } from '@/Explorer/store'
import { uniqueId, type ExplorerItem } from '@/Explorer/types'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { PropsWithChildren, useCallback, type HTMLAttributes } from 'react'
import ExplorerDroppable from '../Draggable/ExplorerDroppable'
import ExplorerDraggable from '../Draggable/ExplorerDraggable'

// see spacedrive's `interface/app/$libraryId/Explorer/View/ViewItem.tsx`

interface ViewItemProps extends PropsWithChildren, HTMLAttributes<HTMLDivElement> {
  isDraggable: boolean
  data: ExplorerItem
}

export default function ViewItem({ data, isDraggable, children, onClick, ...props }: ViewItemProps) {
  const explorerStore = useExplorerStore()
  const explorer = useExplorerContext()
  const explorerViewContext = useExplorerViewContext()

  const handleContextMenuOpenChange = useCallback((open: boolean) => {
    explorerStore.setIsContextMenuOpen(open)
    if (open) {
      if (!explorer.isItemSelected(data)) {
        // 右键菜单出现的时候, 当前条目之前没被选中, 选中当前条目
        explorer.resetSelectedItems([data])
      } else {
        // 右键菜单出现的时候, 当前条目之前已经被选中, 不做任何操作
      }
    }
  }, [explorerStore, explorer, data])

  return (
    <div
      {...props}
      data-selecto-item={uniqueId(data)}
      data-component-hint='ViewItem'
      onClick={(e) => {
        // ExplorerLayout 上面有一个 onClick={resetSelectedItems} 会清空选中的项目, 这里一定要 stop 一下
        e.stopPropagation()
        if (onClick) {
          onClick(e)
        }
      }}
    >
      <ContextMenu.Root onOpenChange={handleContextMenuOpenChange}>
        <ContextMenu.Trigger>
          <ExplorerDroppable droppable={{ data: data }}>
            <ExplorerDraggable draggable={{ data: data, disabled: !isDraggable }}>
              {children}
            </ExplorerDraggable>
          </ExplorerDroppable>
        </ContextMenu.Trigger>
        <ContextMenu.Portal>
          {explorerViewContext.contextMenu && explorerViewContext.contextMenu(data)}
        </ContextMenu.Portal>
      </ContextMenu.Root>
    </div>
  )
}
