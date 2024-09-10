import { useMoveTargetSelected } from '@/hooks/useMoveTargetSelected'
import { useOpenFileSelection } from '@/hooks/useOpenFileSelection'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { BaseContextMenuItem } from './types'

function withMoveExplorerItem(BaseComponent: BaseContextMenuItem) {
  return function ContextMenuMove() {
    const { openFileSelection } = useOpenFileSelection()
    const { onMoveTargetSelected } = useMoveTargetSelected()

    return (
      <ContextMenu.Item onSelect={() => openFileSelection().then((path) => onMoveTargetSelected(path))}>
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withMoveExplorerItem(() => <div>Move</div>)
