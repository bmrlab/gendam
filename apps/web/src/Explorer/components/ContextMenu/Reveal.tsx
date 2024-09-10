import { useExplorerContext } from '@/Explorer/hooks'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useRouter } from 'next/navigation'
import { useCallback } from 'react'
import { BaseContextMenuItem } from './types'

function withRevealExplorerItem(BaseComponent: BaseContextMenuItem) {
  return function ContextMenuReveal() {
    const router = useRouter()
    const explorer = useExplorerContext()

    const handleReveal = useCallback(() => {
      const item = Array.from(explorer.selectedItems).at(0)
      if (!item) {
        return
      }

      const [dir, id] = (() => {
        if (item.type === 'FilePath') {
          return [item.filePath.materializedPath, item.filePath.id]
        }

        if (item.type === 'SearchResult') {
          return [item.filePaths.at(0)?.materializedPath, item.filePaths.at(0)?.id]
        }

        return [void 0, void 0]
      })()

      if (dir && id) {
        router.push(`/explorer?dir=${dir}&id=${id}`)
      }
    }, [router, explorer.selectedItems])

    return (
      <ContextMenu.Item onSelect={handleReveal} disabled={explorer.selectedItems.size > 1}>
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withRevealExplorerItem(() => <div>Reveal in Explorer</div>)
