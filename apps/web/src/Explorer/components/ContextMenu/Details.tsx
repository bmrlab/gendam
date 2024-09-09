import { useExplorerContext } from '@/Explorer/hooks'
import { useInspector } from '@/components/Inspector/store'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useCallback } from 'react'
import { BaseContextMenuItem } from './types'

function withShowInspectorExplorerItem(BaseComponent: BaseContextMenuItem) {
  return () => {
    const explorer = useExplorerContext()
    const inspector = useInspector()

    const handleShowInspector = useCallback(
      (e: Event) => {
        inspector.setShow(true)
      },
      [inspector],
    )

    return (
      <ContextMenu.Item onSelect={handleShowInspector} disabled={explorer.selectedItems.size > 1}>
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withShowInspectorExplorerItem(() => <div>Details</div>)
