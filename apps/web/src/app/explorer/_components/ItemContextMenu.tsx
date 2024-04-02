import { useExplorerContext } from '@/Explorer/hooks'
import { ExplorerItem } from '@/Explorer/types'
import { useExplorerStore } from '@/Explorer/store'
import { rspc } from '@/lib/rspc'
import { ContextMenuPrimitive as ContextMenu } from '@muse/ui/v1/context-menu'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { forwardRef, useCallback } from 'react'
import { twx } from '@/lib/utils'

type ItemContextMenuProps = {
  data: ExplorerItem
}

const ContextMenuItem = twx(ContextMenu.Item)`
relative cursor-default select-none outline-none
focus:bg-accent focus:text-white hover:bg-accent hover:text-white
data-[disabled]:pointer-events-none data-[disabled]:opacity-50
flex cursor-default items-center justify-start rounded-md px-2 py-2 text-sm
`

const ItemContextMenu = forwardRef<typeof ContextMenu.Content, ItemContextMenuProps>(function ItemContextMenuComponent(
  { data, ...prpos },
  forwardedRef,
) {
  const router = useRouter()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const deleteMut = rspc.useMutation(['assets.delete_file_path'])
  const metadataMut = rspc.useMutation(['assets.process_video_metadata'])

  /**
   * 这里都改成处理 selectedItems 而不只是处理当前的 item
   * ViewItem.tsx 里面的 handleContextMenuOpenChange 已经确保了当前 item 在 selectedItems 里
   */

  const handleOpen = useCallback(
    (e: Event) => {
      // e.stopPropagation()
      if (!explorer.parentPath) {
        return
      }
      explorer.resetSelectedItems()
      explorerStore.reset()
      if (data.isDir) {
        let newPath = explorer.parentPath + data.name + '/'
        router.push('/explorer?dir=' + newPath)
      } else {
        // do nothing, for the moment
      }
    },
    [data, explorer, router, explorerStore],
  )

  const handleDelete = useCallback(
    (e: Event) => {
      // e.stopPropagation()
      if (!explorer.parentPath) {
        return
      }
      for (let item of Array.from(explorer.selectedItems)) {
        deleteMut.mutate({
          path: explorer.parentPath,
          name: item.name,
        })
      }
      explorer.resetSelectedItems()
    },
    [deleteMut, explorer],
  )

  const handleRename = useCallback(
    (e: Event) => {
      // e.stopPropagation()
      if (!explorer.parentPath) {
        return
      }
      explorerStore.setIsRenaming(true)
    },
    [explorer, explorerStore],
  )

  const handleProcessMetadata = useCallback(
    (e: Event) => {
      // e.stopPropagation()
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
    <ContextMenu.Content
      ref={forwardedRef as any}
      className="w-60 rounded-md text-ink bg-app-box border border-app-line p-1 shadow-lg"
      {...prpos}
      onClick={(e) => e.stopPropagation()}
    >
      <ContextMenuItem onSelect={handleOpen} disabled={explorer.selectedItems.size > 1 }>
        <div className="mx-1 truncate text-xs">打开</div>
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => explorerStore.setIsFoldersDialogOpen(true)}>
        <div className="mx-1 truncate text-xs">移动</div>
      </ContextMenuItem>
      <ContextMenuItem onSelect={handleProcessMetadata}>
        <div className="mx-1 truncate text-xs">刷新视频信息</div>
      </ContextMenuItem>
      <ContextMenuItem onSelect={() => {}} disabled={explorer.selectedItems.size > 1 }>
        <div className="mx-1 truncate text-xs">预览</div>
      </ContextMenuItem>
      <ContextMenuItem onSelect={handleRename} disabled={explorer.selectedItems.size > 1 }>
        <div className="mx-1 truncate text-xs">重命名</div>
      </ContextMenuItem>
      <ContextMenuItem
        className={classNames('text-red-600 focus:bg-red-500/90 focus:text-white hover:bg-red-500/90 hover:text-white')}
        onSelect={handleDelete}
      >
        <div className="mx-1 truncate text-xs">删除</div>
      </ContextMenuItem>
    </ContextMenu.Content>
  )
})

export default ItemContextMenu
