'use client'
import { useExplorerContext, useExplorerViewContext } from '@/Explorer/hooks'
import { useExplorerStore } from '@/Explorer/store'
import { type ExplorerItem } from '@/Explorer/types'
import { ContextMenuPortal, ContextMenuRoot, ContextMenuTrigger } from '@muse/ui/v1/context-menu'
import { PropsWithChildren, useCallback, type HTMLAttributes } from 'react'

// see spacedrive's `interface/app/$libraryId/Explorer/View/ViewItem.tsx`

interface ViewItemProps extends PropsWithChildren, HTMLAttributes<HTMLDivElement> {
  data: ExplorerItem
}

export default function ViewItem({ data, children, ...props }: ViewItemProps) {
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
    <ContextMenuRoot onOpenChange={handleContextMenuOpenChange}>
      <ContextMenuTrigger>
        {children}
      </ContextMenuTrigger>
      <ContextMenuPortal>
        {explorerViewContext.contextMenu && explorerViewContext.contextMenu(data)}
      </ContextMenuPortal>
    </ContextMenuRoot>
  )
}
