import { useExplorerContext } from '@/Explorer/hooks'
import { useExplorerStore } from '@/Explorer/store'
import { type ExplorerItem } from '@/Explorer/types'
import { useInspector } from '@/components/Inspector/store'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import { type FilePath } from '@/lib/bindings'
import { queryClient, rspc } from '@/lib/rspc'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useRouter } from 'next/navigation'
import { forwardRef, useCallback, useMemo } from 'react'

type ItemContextMenuProps = {
  data: FilePath
}

const ItemContextMenu = forwardRef<typeof ContextMenu.Content, ItemContextMenuProps>(function ItemContextMenuComponent(
  { data, ...prpos },
  forwardedRef,
) {
  const router = useRouter()

  // Explorer Component's State and Context
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const selectedFilePathItems = useMemo(() => {
    type T = Extract<ExplorerItem, { type: 'FilePath' }>
    const filtered = Array.from(explorer.selectedItems).filter((item) => item.type === 'FilePath') as T[]
    return filtered.map((item) => item.filePath)
  }, [explorer.selectedItems])

  // Page Specific State and Context
  const inspector = useInspector()

  // Shared State and Context
  const quickViewStore = useQuickViewStore()

  const deleteMut = rspc.useMutation(['assets.delete_trash_file_path'])

  const putBackMut = rspc.useMutation(['assets.put_back'])

  const handleOpen = useCallback(
    (e: Event) => {
      // e.stopPropagation()
      explorer.resetSelectedItems()
      explorerStore.reset()
      if (data.isDir) {
        let newPath = data.materializedPath + data.name + '/'
        router.push(location.pathname + '?dir=' + newPath)
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
    async (e: Event) => {
      for (let item of selectedFilePathItems) {
        try {
          await deleteMut.mutateAsync({
            materializedPath: item.materializedPath,
            name: item.name,
          })
        } catch (error) {}
        queryClient.invalidateQueries({
          queryKey: ['assets.trash', { materializedPath: item.materializedPath }],
        })
      }
      explorer.resetSelectedItems()
    },
    [selectedFilePathItems, deleteMut, explorer],
  )

  const handlePutBack = useCallback(async () => {
    for (let item of selectedFilePathItems) {
      try {
        await putBackMut.mutateAsync({
          materializedPath: item.materializedPath,
          name: item.name,
        })
      } catch (error) {}
      queryClient.invalidateQueries({
        queryKey: ['assets.trash', { materializedPath: item.materializedPath }],
      })
    }
    explorer.resetSelectedItems()
  }, [selectedFilePathItems, deleteMut, explorer])

  return (
    <ContextMenu.Content
      ref={forwardedRef as any}
      {...prpos}
      onClick={(e) => e.stopPropagation()}
      className="data-[state=closed]:animate-none data-[state=closed]:duration-0"
    >
      <ContextMenu.Item onSelect={handleOpen} disabled={explorer.selectedItems.size > 1}>
        <div>Open</div>
      </ContextMenu.Item>
      <ContextMenu.Item onSelect={handleShowInspector} disabled={explorer.selectedItems.size > 1}>
        <div>Details</div>
      </ContextMenu.Item>
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenu.Item onSelect={handlePutBack}>
        <div>Put back</div>
      </ContextMenu.Item>
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenu.Item variant="destructive" onSelect={handleDelete}>
        <div>Delete</div>
      </ContextMenu.Item>
    </ContextMenu.Content>
  )
})

export default ItemContextMenu
