import { useExplorerContext } from '@/Explorer/hooks'
import { queryClient, rspc } from '@/lib/rspc'
import { confirm } from '@/lib/utils'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useCallback, useMemo } from 'react'
import { BaseContextMenuItem } from './types'

function withRebuildIndexExplorerItem(BaseComponent: BaseContextMenuItem) {
  return function ContextMenuReprocess() {
    const explorer = useExplorerContext()

    const rebuildIndexMut = rspc.useMutation(['assets.rebuild_content_index'])

    const validAssetObjects = useMemo(() => {
      return Array.from(explorer.selectedItems)
        .map((v) => ('assetObject' in v ? v.assetObject : void 0))
        .filter((v) => !!v)
    }, [explorer.selectedItems])

    const handleReprocess = useCallback(
      async (e: Event) => {
        const withExistingArtifacts = await confirm({
          title: 'Rebuild Index',
          description: 'Keep existing completed tasks or reprocess from beginning?',
          confirmText: 'Keep Existing',
          cancelText: 'Reprocess All',
        })
        for (let item of validAssetObjects) {
          try {
            await rebuildIndexMut.mutateAsync({ assetObjectHash: item.hash, withExistingArtifacts })
          } catch (error) {}
          queryClient.invalidateQueries({
            queryKey: ['tasks.list', { filter: { assetObjectId: item.id } }],
          })
        }
      },
      [validAssetObjects, rebuildIndexMut],
    )

    return (
      <ContextMenu.Item onSelect={handleReprocess} disabled={explorer.selectedItems.size > validAssetObjects.length}>
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withRebuildIndexExplorerItem(() => <div>Rebuild index</div>)
