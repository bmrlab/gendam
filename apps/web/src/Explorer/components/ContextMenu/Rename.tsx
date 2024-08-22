import { useExplorerContext } from '@/Explorer/hooks'
import { useExplorerStore } from '@/Explorer/store'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useCallback } from 'react'
import { BaseContextMenuItem } from './types'

function withRenameExplorerItem(BaseComponent: BaseContextMenuItem) {
  return () => {
    const explorer = useExplorerContext()
    const explorerStore = useExplorerStore()

    const handleRename = useCallback(
      (e: Event) => {
        explorerStore.setIsRenaming(true)
      },
      [explorerStore],
    )

    return (
      <ContextMenu.Item onSelect={handleRename} disabled={explorer.selectedItems.size > 1}>
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withRenameExplorerItem(() => <div>Rename</div>)
