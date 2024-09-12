import { useExplorerContext } from '@/Explorer/hooks'
import { queryClient, rspc } from '@/lib/rspc'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useCallback, useMemo } from 'react'
import { BaseContextMenuItem } from './types'

function withDeleteExplorerItem(BaseComponent: BaseContextMenuItem) {
  return function ContextMenuDelete() {
    const deleteMut = rspc.useMutation(['assets.delete_file_path'])
    const explorer = useExplorerContext()

    const selectedFilePathItems = useMemo(() => {
      return Array.from(explorer.selectedItems).filter(
        (item) => item.type === 'FilePathDir' || item.type === 'FilePathWithAssetObject',
      )
    }, [explorer.selectedItems])

    const handleDelete = useCallback(
      async (e: Event) => {
        for (let item of selectedFilePathItems) {
          try {
            await deleteMut.mutateAsync({
              materializedPath: item.filePath.materializedPath,
              name: item.filePath.name,
            })
          } catch (error) {}
          queryClient.invalidateQueries({
            queryKey: ['assets.list', { materializedPath: item.filePath.materializedPath }],
          })
        }
        explorer.resetSelectedItems()
      },
      [selectedFilePathItems, deleteMut, explorer],
    )

    return (
      <ContextMenu.Item variant="destructive" onSelect={handleDelete}>
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withDeleteExplorerItem(() => <div>Delete</div>)
