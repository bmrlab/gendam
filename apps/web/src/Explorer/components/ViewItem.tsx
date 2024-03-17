'use client'
import { useExplorerContext, useExplorerViewContext } from '@/Explorer/hooks'
import { useExplorerStore } from '@/Explorer/store'
import { type ExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
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

  const processVideoMut = rspc.useMutation(['assets.process_video_asset'])

  const doubleClick = useCallback(() => {
    explorer.resetSelectedItems()
    if (data.isDir) {
      let newPath = explorer.parentPath + data.name + '/'
      router.push('/explorer?dir=' + newPath)
    } else {
      // processVideoMut.mutate(data.id)
      router.push('/video-tasks')
    }
  }, [data, explorer, router])

  return (
    <ContextMenuRoot onOpenChange={(open) => explorerStore.setIsContextMenuOpen(open)}>
      <ContextMenuTrigger>
        <div
          {...props}
          onDoubleClick={(e) => {
            // e.stopPropagation()
            doubleClick()
          }}
        >
          {children}
        </div>
      </ContextMenuTrigger>
      <ContextMenuPortal>{explorerViewContext.contextMenu}</ContextMenuPortal>
    </ContextMenuRoot>
  )
}
