import { useExplorerContext } from '@/Explorer/hooks'
import { ExplorerItem } from '@/Explorer/types'
import { useExplorerStore } from '@/Explorer/store'
import { rspc } from '@/lib/rspc'
import { ContextMenuContent, ContextMenuItem } from '@muse/ui/v1/context-menu'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { forwardRef, useCallback } from 'react'
import { twx } from '@/lib/utils'

type ItemContextMenuProps = {
  data: ExplorerItem
}

const _MenuItemDefault = twx(ContextMenuItem)`flex cursor-default items-center justify-start rounded-md px-2 py-2 hover:bg-neutral-200/60`

const ItemContextMenu = forwardRef<typeof ContextMenuContent, ItemContextMenuProps>(function ItemContextMenuComponent(
  { data, ...prpos },
  forwardedRef,
) {
  const router = useRouter()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const deleteMut = rspc.useMutation(['assets.delete_file_path'])
  const metadataMut = rspc.useMutation(['assets.process_video_metadata'])

  const handleOpen = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      e.stopPropagation()
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
    (e: React.FormEvent<HTMLDivElement>) => {
      e.stopPropagation()
      if (!explorer.parentPath) {
        return
      }
      explorer.resetSelectedItems()
      deleteMut.mutate({
        path: explorer.parentPath,
        name: data.name,
      })
    },
    [deleteMut, explorer, data],
  )

  const handleRename = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      e.stopPropagation()
      if (!explorer.parentPath) {
        return
      }
      explorerStore.setIsRenaming(true)
    },
    [explorer, explorerStore],
  )

  const handleProcessMetadata = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      e.stopPropagation()
      if (!data.assetObject) {
        return
      }
      metadataMut.mutate(data.assetObject.id)
    },
    [metadataMut, data],
  )

  return (
    <ContextMenuContent
      ref={forwardedRef as any}
      className="w-60 rounded-md border border-neutral-100 bg-white p-1 shadow-lg"
      {...prpos}
    >
      <_MenuItemDefault onClick={handleOpen}>
        <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">打开</div>
      </_MenuItemDefault>
      <_MenuItemDefault onClick={() => explorerStore.setIsFoldersDialogOpen(true)}>
        <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">移动</div>
      </_MenuItemDefault>
      <_MenuItemDefault onClick={handleProcessMetadata}>
        <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">刷新视频信息</div>
      </_MenuItemDefault>
      <_MenuItemDefault onClick={() => {}}>
        <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">预览</div>
      </_MenuItemDefault>
      <_MenuItemDefault onClick={handleRename}>
        <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">重命名</div>
      </_MenuItemDefault>
      <ContextMenuItem
        className={classNames(
          'flex cursor-default items-center justify-start rounded-md px-2 py-2',
          'text-red-600 hover:bg-red-500/90 hover:text-white',
        )}
        onClick={handleDelete}
      >
        <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">删除</div>
      </ContextMenuItem>
    </ContextMenuContent>
  )
})

export default ItemContextMenu
