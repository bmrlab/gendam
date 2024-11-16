import { useExplorerContext } from '@/Explorer/hooks'
import { queryClient, rspc } from '@/lib/rspc'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useCallback, useMemo } from 'react'
import { BaseContextMenuItem } from './types'

function withProcessMetadataExplorerItem(BaseComponent: BaseContextMenuItem) {
  return function ContextMenuProcessMetadata() {
    const explorer = useExplorerContext()
    const metadataMut = rspc.useMutation(['assets.process_asset_metadata'])

    const validAssetObjects = useMemo(() => {
      return Array.from(explorer.selectedItems)
        .map((v) => ('assetObject' in v ? v.assetObject : void 0))
        .filter((v) => !!v)
    }, [explorer.selectedItems])

    const handleProcessMetadata = useCallback(
      async (e: Event) => {
        for (let item of validAssetObjects) {
          try {
            await metadataMut.mutateAsync(item.hash)
          } catch (error) {}
          queryClient.invalidateQueries({
            queryKey: ['assets.list', { materializedPath: explorer.materializedPath }],
          })
        }
      },
      [validAssetObjects, metadataMut, explorer.materializedPath],
    )

    return (
      <ContextMenu.Item
        onSelect={handleProcessMetadata}
        disabled={explorer.selectedItems.size > validAssetObjects.length}
      >
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withProcessMetadataExplorerItem(() => <div>Regen thumbnail</div>)
