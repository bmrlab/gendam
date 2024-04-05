import { create } from 'zustand'
import { ExplorerItem } from '../types'

type DragState = {
  type: 'dragging'
  items: ExplorerItem[]
}

interface ExplorerState {
  drag: null | DragState
  setDrag: (drag: DragState|null) => void

  isRenaming: boolean
  setIsRenaming: (isRenaming: boolean) => void

  isContextMenuOpen: boolean
  setIsContextMenuOpen: (isContextMenuOpen: boolean) => void

  reset: () => void
}

export const useExplorerStore = create<ExplorerState>((set) => ({
  drag: null,
  setDrag: (drag) => set({ drag }),

  isRenaming: false,
  setIsRenaming: (isRenaming) => set({ isRenaming }),

  isContextMenuOpen: false,
  setIsContextMenuOpen: (isContextMenuOpen) => set({ isContextMenuOpen }),

  reset: () => set({ isRenaming: false, isContextMenuOpen: false, drag: null }),
}))
