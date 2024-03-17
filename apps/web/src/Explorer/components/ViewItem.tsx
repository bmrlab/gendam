'use client'
import { useExplorerContext, useExplorerViewContext } from '@/Explorer/hooks'
import { useExplorerStore } from '@/Explorer/store'
import { type ExplorerItem } from '@/Explorer/types'
// import { rspc } from '@/lib/rspc'
import { ContextMenuPortal, ContextMenuRoot, ContextMenuTrigger } from '@muse/ui/v1/context-menu'
import { useRouter } from 'next/navigation'
import { PropsWithChildren, useCallback, type HTMLAttributes } from 'react'

// see spacedrive's `interface/app/$libraryId/Explorer/View/ViewItem.tsx`

interface ViewItemProps extends PropsWithChildren, HTMLAttributes<HTMLDivElement> {
  data: ExplorerItem
}

export default function ViewItem({ data, children, ...props }: ViewItemProps) {
  const router = useRouter()
  const explorerStore = useExplorerStore()
  const explorer = useExplorerContext()
  const explorerViewContext = useExplorerViewContext()

  // const processVideoMut = rspc.useMutation(['assets.process_video_asset'])
  const handleDoubleClick = useCallback((e: React.FormEvent<HTMLDivElement>) => {
    // e.stopPropagation()
    explorer.resetSelectedItems()
    explorerStore.reset()
    if (data.isDir) {
      let newPath = explorer.parentPath + data.name + '/'
      router.push('/explorer?dir=' + newPath)
    } else {
      // processVideoMut.mutate(data.id)
      router.push('/video-tasks')
    }
  }, [data, explorer, router, explorerStore])

  const handleContextMenuOpenChange = useCallback((open: boolean) => {
    explorerStore.setIsContextMenuOpen(open)
    if (open) {
      // 右键菜单出现的时候，同时也选中触发它的 item
      explorer.resetSelectedItems([data])
    }
  }, [explorerStore, explorer, data])

  return (
    <ContextMenuRoot onOpenChange={handleContextMenuOpenChange}>
      <ContextMenuTrigger>
        <div {...props} onDoubleClick={handleDoubleClick}>
          {children}
        </div>
      </ContextMenuTrigger>
      <ContextMenuPortal>
        {explorerViewContext.contextMenu && explorerViewContext.contextMenu(data)}
      </ContextMenuPortal>
    </ContextMenuRoot>
  )
}
