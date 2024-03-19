import { ExplorerItem } from '@/Explorer/types'
import { useDroppable, UseDroppableArguments } from '@dnd-kit/core'
import { HTMLAttributes } from 'react'

export interface UseExplorerDroppableProps extends Omit<UseDroppableArguments, 'id'> {
  data: ExplorerItem
}

/**
 * TODO: droppable 要增加一个属性
 * disabled: (!isFolder && !isLocation) || props.selected
 * 被选中的或者不是文件夹的不能被 drop
 */

const ExplorerDroppable = ({
  droppable,
  children,
}: HTMLAttributes<HTMLDivElement> & {
  droppable: UseExplorerDroppableProps
}) => {
  const { isOver, setNodeRef } = useDroppable({
    id: droppable.data.id.toString(),
    data: droppable.data,
  })

  return <div ref={setNodeRef}>{children}</div>
}

export default ExplorerDroppable
