import { ExplorerItem } from '@/Explorer/types'
import { useExplorerStore } from '@/Explorer/store'
import { useDroppable, UseDroppableArguments } from '@dnd-kit/core'
import { HTMLAttributes, useMemo } from 'react'

export interface UseExplorerDroppableProps extends Omit<UseDroppableArguments, 'id'> {
  data: ExplorerItem
}

const ExplorerDroppable = ({
  droppable,
  children,
}: HTMLAttributes<HTMLDivElement> & {
  droppable: UseExplorerDroppableProps
}) => {
  const explorerStore = useExplorerStore()

  const itemIsBeingDragged = useMemo<boolean>(() => {
    if (explorerStore.drag?.type === 'dragging') {
      return explorerStore.drag?.items.some((item) => item.id === droppable.data.id) || false
    } else {
      return false
    }
  }, [droppable, explorerStore])

  const { isOver, setNodeRef } = useDroppable({
    id: droppable.data.id.toString(),
    data: droppable.data,
    disabled: !droppable.data.isDir || itemIsBeingDragged,
  })

  return <div ref={setNodeRef}>{children}</div>
}

export default ExplorerDroppable
