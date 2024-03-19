import { ExplorerItem } from '@/Explorer/types'
import { useDroppable, UseDroppableArguments } from '@dnd-kit/core'
import { HTMLAttributes } from 'react'

export interface UseExplorerDroppableProps extends Omit<UseDroppableArguments, 'id'> {
  data: ExplorerItem
}

const ExplorerDroppable = ({
  droppable,
  children,
}: HTMLAttributes<HTMLDivElement> & {
  droppable: UseExplorerDroppableProps
}) => {
  const { isOver, setNodeRef } = useDroppable({
    id: droppable.data.id.toString(),
  })

  return <div ref={setNodeRef}>{children}</div>
}

export default ExplorerDroppable
