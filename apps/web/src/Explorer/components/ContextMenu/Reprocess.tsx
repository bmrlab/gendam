import { useExplorerContext } from '@/Explorer/hooks'
import { queryClient, rspc } from '@/lib/rspc'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useCallback, useMemo } from 'react'
import { BaseContextMenuItem } from './types'

function withReprocessExplorerItem(BaseComponent: BaseContextMenuItem) {
  return () => {
    const explorer = useExplorerContext()

    const processJobsMut = rspc.useMutation(['video.tasks.regenerate'])

    const validAssetObjects = useMemo(() => {
      return Array.from(explorer.selectedItems)
        .map((v) => ('assetObject' in v ? v.assetObject : void 0))
        .filter((v) => !!v)
    }, [explorer.selectedItems])

    const handleReprocess = useCallback(
      async (e: Event) => {
        for (let item of validAssetObjects) {
          try {
            await processJobsMut.mutateAsync({ assetObjectId: item.id })
          } catch (error) {}
          queryClient.invalidateQueries({
            queryKey: ['tasks.list', { filter: { assetObjectId: item.id } }],
          })
        }
      },
      [validAssetObjects, processJobsMut],
    )

    return (
      <ContextMenu.Item onSelect={handleReprocess} disabled={explorer.selectedItems.size > 1}>
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withReprocessExplorerItem(() => <div>Re-process jobs</div>)
