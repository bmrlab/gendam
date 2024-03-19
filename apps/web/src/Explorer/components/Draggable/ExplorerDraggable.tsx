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
  draggable: UseExplorerDraggableProps
}) => {
  const { attributes, listeners, setNodeRef, transform } = useDraggable({
    id: draggable.data.id.toString(),
    data: draggable.data,
    // disabled: true,  // TODO: 有些时候是不能 drag 的，这里要小心判断
  })

  // attributes.role 默认是 button, 浏览器自带样式 cursor: pointer
  const style: {[key:string]:string} = {
    cursor: 'default'
  }

  // if (transform) {
  //   // style.transform = `translate3d(${transform.x}px, ${transform.y}px, 0)`
  //   // style.transform = 'translate3d(0, 0, 0)'
  //   style.transform = 'None'
  // }

  return (
    <div ref={setNodeRef} style={style} {...listeners} {...attributes}>
      {children}
    </div>
  )
}

export default ExplorerDraggable
