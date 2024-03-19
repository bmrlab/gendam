import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import { useDroppable, UseDroppableArguments } from '@dnd-kit/core'
import { createContext, HTMLAttributes, useContext, useMemo } from 'react'

export interface UseExplorerDroppableProps extends Omit<UseDroppableArguments, 'id'> {
  data: ExplorerItem
}

const ExplorerDroppableContext = createContext<{ isDroppable: boolean } | null>(null)
export const useExplorerDroppableContext = () => {
  const ctx = useContext(ExplorerDroppableContext)
  if (ctx === null) throw new Error('ExplorerDroppableContext.Provider not found!')
  return ctx
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

  /**
   * isOver 是指当前的是否拖动到了此元素上
   * TODO: 需要判断当前 data 的类型，比如，是一个 folder，而不是简单的 isDroppable = isOver
   */
  const isDroppable = useMemo(() => isOver, [isOver])

  const context = useMemo(() => ({ isDroppable }), [isDroppable]);

  return (
    // 把 isDroppable (即 Droppable 的 isOver 等信息带到子元素中去)
    <ExplorerDroppableContext.Provider value={context}>
      <div ref={setNodeRef} data-component='ExplorerDroppable'>{children}</div>
    </ExplorerDroppableContext.Provider>
  )
}

export default ExplorerDroppable
