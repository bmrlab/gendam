import { ExplorerItem } from '@/Explorer/types'
import { useDraggable, UseDraggableArguments } from '@dnd-kit/core'
import { HTMLAttributes } from 'react'

export interface UseExplorerDraggableProps extends Omit<UseDraggableArguments, 'id'> {
  data: ExplorerItem
}

const ExplorerDraggable = ({
  draggable,
  children,
}: Omit<HTMLAttributes<HTMLDivElement>, 'draggable'> & {
	draggable: UseExplorerDraggableProps;
}) => {
  const { attributes, listeners, setNodeRef, transform } = useDraggable({
    id: draggable.data.id.toString(),
  })

  const style = transform
    ? {
        transform: `translate3d(${transform.x}px, ${transform.y}px, 0)`,
      }
    : undefined

  return (
    <div ref={setNodeRef} style={style} {...listeners} {...attributes}>
      {children}
    </div>
  )
}

export default ExplorerDraggable
