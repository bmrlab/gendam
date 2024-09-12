import { useExplorerContext } from '@/Explorer/hooks'
import { useExplorerStore } from '@/Explorer/store'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useRouter } from 'next/navigation'
import { useCallback } from 'react'
import { BaseContextMenuItem } from './types'

function withOpenExplorerItem(BaseComponent: BaseContextMenuItem) {
  return function ContextMenuOpen() {
    const router = useRouter()
    const explorer = useExplorerContext()
    const explorerStore = useExplorerStore()
    const quickViewStore = useQuickViewStore()

    const handleOpen = useCallback(
      (e: Event) => {
        // e.stopPropagation()
        const data = Array.from(explorer.selectedItems).at(0)

        if (!data) return

        explorer.resetSelectedItems()
        explorerStore.reset()
        if (data.type === 'FilePath' && data.filePath.isDir) {
          let newPath = data.filePath.materializedPath + data.filePath.name + '/'
          router.push('/explorer?dir=' + newPath)
        } else if (data.type !== 'Unknown' && data.type !== 'LibraryRoot' && data.assetObject) {
          quickViewStore.open(data)
        }
      },
      [explorer, router, explorerStore, quickViewStore],
    )

    return (
      <ContextMenu.Item onSelect={handleOpen} disabled={explorer.selectedItems.size > 1}>
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withOpenExplorerItem(() => <div>Open</div>)
