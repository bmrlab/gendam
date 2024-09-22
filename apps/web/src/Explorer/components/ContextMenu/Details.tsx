import { useExplorerContext } from '@/Explorer/hooks'
import { useInspector } from '@/components/Inspector'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useCallback } from 'react'
import { BaseContextMenuItem } from './types'

function withShowInspectorExplorerItem(BaseComponent: BaseContextMenuItem) {
  // 这个菜单要确保在 InspectorProvider 里面，否则无法显示 Inspector 并且运行就会报错
  // 具体看 apps/web/src/components/Inspector/hooks.tsx#L95-L96
  return function ContextMenuDetails() {
    const explorer = useExplorerContext()
    const inspector = useInspector()

    const handleShowInspector = useCallback(
      (e: Event) => {
        // 这里不保存设置，只是临时显示一下 inspector
        // 如果这个 ContextMenu 没有在 InspectorContext 里面，这个 setShow 无效，符合预期
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
