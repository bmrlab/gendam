import { useExplorerStore } from '@/Explorer/store'
import { uniqueId, type ExplorerItem } from '@/Explorer/types'
import { useDroppable, UseDroppableArguments } from '@dnd-kit/core'
import { createContext, HTMLAttributes, useContext, useMemo } from 'react'

export interface UseExplorerDroppableProps extends Omit<UseDroppableArguments, 'id'> {
  region?: 'Sidebar' | 'Toolbar' | 'StatusBar' // see below for more info
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
      return explorerStore.drag?.items.some((item) => uniqueId(item) === uniqueId(droppable.data)) || false
    } else {
      return false
    }
  }, [droppable, explorerStore])

  /**
   * 给 id 加一个 region, 确保不同地方的文件夹有不同的 id，因为相同 id 的 droppable 只有一个会被 DND 使用
   * 这个 id 只给 DND 组件用，在其他地方都直接用 data 的 id
   */
  const id = `${uniqueId(droppable.data)}`
  const isDir = droppable.data.type === 'LibraryRoot' || droppable.data.type === 'FilePathDir'
  const { isOver, setNodeRef } = useDroppable({
    id: droppable.region ? `${droppable.region}/${id}` : id,
    data: droppable.data,
    disabled: itemIsBeingDragged || !isDir,
  })

  /**
   * isOver 是指当前的是否拖动到了此元素上
   * TODO: 需要判断当前 data 的类型，比如，是一个 folder，而不是简单的 isDroppable = isOver
   */
  const isDroppable = useMemo(() => isOver, [isOver])

  const context = useMemo(() => ({ isDroppable }), [isDroppable])

  return (
    // 把 isDroppable (即 Droppable 的 isOver 等信息带到子元素中去)
    <ExplorerDroppableContext.Provider value={context}>
      <div ref={setNodeRef} data-component-hint="ExplorerDroppable">
        {children}
      </div>
    </ExplorerDroppableContext.Provider>
  )
}

export default ExplorerDroppable
