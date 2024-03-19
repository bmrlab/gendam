import { create } from 'zustand'
import { ExplorerItem } from '../types'

type DragState = {
  type: 'dragging'
  items: ExplorerItem[]
}

interface ExplorerState {
  drag: null | DragState
  isDraggingSelected: boolean // 正在拖拽选中的项目
  isRenaming: boolean
  isContextMenuOpen: boolean
  setDrag: (items: ExplorerItem[]) => void
  setIsRenaming: (isRenaming: boolean) => void
  setIsContextMenuOpen: (isContextMenuOpen: boolean) => void
  reset: () => void
}

export const useExplorerStore = create<ExplorerState>((set) => ({
  drag: null,
  isDraggingSelected: false,
  isRenaming: false,
  isContextMenuOpen: false,
  setDrag: (items: ExplorerItem[]) => {
    if (items.length === 0) {
      set({
        drag: null,
        isDraggingSelected: false,
      })
    } else {
      set({
        drag: {
          type: 'dragging',
          items,
        },
        isDraggingSelected: true,
      })
    }
  },
  setIsRenaming: (isRenaming) => set({ isRenaming }),
  setIsContextMenuOpen: (isContextMenuOpen) => set({ isContextMenuOpen }),
  reset: () => set({ isRenaming: false, isContextMenuOpen: false }),
}))
