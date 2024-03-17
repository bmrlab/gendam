'use client'
import { useAssetContextMenu } from '@/Explorer/AssetContextMenu/Context'
import { useExplorerContext } from '@/Explorer/Context'
import { useExplorerViewContext } from '@/Explorer/hooks'
import { useExplorerStore } from '@/Explorer/store'
import { type ExplorerItem } from '@/Explorer/types'
import { ContextMenuPortal, ContextMenuRoot, ContextMenuTrigger } from '@muse/ui/v1/context-menu'
import { PropsWithChildren, type HTMLAttributes } from 'react'

// see spacedrive's `interface/app/$libraryId/Explorer/View/ViewItem.tsx`

interface ViewItemProps extends PropsWithChildren, HTMLAttributes<HTMLDivElement> {
  data: ExplorerItem
}

export default function ViewItem({ data, children, ...props }: ViewItemProps) {
  const explorerStore = useExplorerStore()
  const explorer = useExplorerContext()
  const explorerViewContext = useExplorerViewContext()
  const assetContextMenu = useAssetContextMenu()

  return (
    <ContextMenuRoot onOpenChange={(open) => explorerStore.setIsContextMenuOpen(open)}>
      <ContextMenuTrigger>
        <div
          {...props}
          onDoubleClick={(e) => {
            // e.stopPropagation()
            assetContextMenu.onDoubleClick(data)
            explorer.resetSelectedItems()
          }}
        >
          {children}
        </div>
      </ContextMenuTrigger>
      <ContextMenuPortal>{explorerViewContext.contextMenu}</ContextMenuPortal>
    </ContextMenuRoot>
  )
}
