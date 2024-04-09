import { useExplorerContext } from '@/Explorer/hooks'
import { ExplorerItem } from '@/Explorer/types'
import { useExplorerStore } from '@/Explorer/store'
import { rspc } from '@/lib/rspc'
import { ContextMenu } from '@muse/ui/v2/context-menu'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { forwardRef, useCallback } from 'react'
import { twx } from '@/lib/utils'
import { useInspector } from './Inspector'
import { useFoldersDialog } from './FoldersDialog'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'

type ItemContextMenuProps = {
  data: ExplorerItem
}

const ItemContextMenu = forwardRef<typeof ContextMenu.Content, ItemContextMenuProps>(function ItemContextMenuComponent(
  { data, ...prpos },
  forwardedRef,
) {
  const router = useRouter()

  // Explorer Component's State and Context
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  // Page Specific State and Context
  const inspector = useInspector()
  const foldersDialog = useFoldersDialog()

  // Shared State and Context
  const quickViewStore = useQuickViewStore()

  const deleteMut = rspc.useMutation(['assets.delete_file_path'])
  const metadataMut = rspc.useMutation(['assets.process_video_metadata'])

  /**
   * 这里都改成处理 selectedItems 而不只是处理当前的 item
   * ViewItem.tsx 里面的 handleContextMenuOpenChange 已经确保了当前 item 在 selectedItems 里
   */

  const handleOpen = useCallback(
    (e: Event) => {
      // e.stopPropagation()
      explorer.resetSelectedItems()
      explorerStore.reset()
      if (data.isDir) {
        let newPath = data.materializedPath + data.name + '/'
        router.push('/explorer?dir=' + newPath)
      } else if (data.assetObject) {
        const { name, assetObject } = data
        quickViewStore.open({ name, assetObject })
      }
    },
    [data, explorer, router, explorerStore, quickViewStore],
  )

  const handleShowInspector = useCallback(
    (e: Event) => {
      inspector.setShow(true)
    },
    [inspector],
  )

  const handleDelete = useCallback(
    (e: Event) => {
      for (let item of Array.from(explorer.selectedItems)) {
        deleteMut.mutate({
          materializedPath: item.materializedPath,
          name: item.name,
        })
      }
      explorer.resetSelectedItems()
    },
    [deleteMut, explorer],
  )

  const handleRename = useCallback(
    (e: Event) => {
      explorerStore.setIsRenaming(true)
    },
    [explorerStore],
  )

  const handleProcessMetadata = useCallback(
    (e: Event) => {
      for (let item of Array.from(explorer.selectedItems)) {
        if (!item.assetObject) {
          return
        }
        metadataMut.mutate(item.assetObject.id)
      }
    },
    [metadataMut, explorer],
  )

  return (
    <ContextMenu.Content ref={forwardedRef as any} {...prpos} onClick={(e) => e.stopPropagation()}>
      <ContextMenu.Item onSelect={handleOpen} disabled={explorer.selectedItems.size > 1 }>
        <div>打开</div>
      </ContextMenu.Item>
      <ContextMenu.Item onSelect={handleShowInspector} disabled={explorer.selectedItems.size > 1 }>
        <div>查看详情</div>
      </ContextMenu.Item>
      {/* <ContextMenu.Item onSelect={() => {}} disabled={explorer.selectedItems.size > 1 }>
        <div>预览</div>
      </ContextMenu.Item> */}
      <ContextMenu.Separator className='h-px bg-app-line my-1' />
      <ContextMenu.Item onSelect={handleProcessMetadata}>
        <div>刷新视频信息</div>
      </ContextMenu.Item>
      <ContextMenu.Item onSelect={() => foldersDialog.setOpen(true)}>
        <div>移动</div>
      </ContextMenu.Item>
      <ContextMenu.Item onSelect={handleRename} disabled={explorer.selectedItems.size > 1 }>
        <div>重命名</div>
      </ContextMenu.Item>
      <ContextMenu.Separator className='h-px bg-app-line my-1' />
      <ContextMenu.Item
        variant='destructive'
        onSelect={handleDelete}
      >
        <div>删除</div>
      </ContextMenu.Item>
    </ContextMenu.Content>
  )
})

export default ItemContextMenu
