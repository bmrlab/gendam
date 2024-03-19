import { create } from 'zustand'
import { ExplorerItem } from '../types'

type DragState = {
  type: 'dragging'
  items: ExplorerItem[]
}

interface ExplorerState {
  drag: null | DragState
  isRenaming: boolean
  isContextMenuOpen: boolean
  setDrag: (drag: DragState|null) => void
  setIsRenaming: (isRenaming: boolean) => void
  setIsContextMenuOpen: (isContextMenuOpen: boolean) => void
  reset: () => void
}

export const useExplorerStore = create<ExplorerState>((set) => ({
  drag: null,
  isRenaming: false,
  isContextMenuOpen: false,
  setDrag: (drag) => set({ drag }),
  setIsRenaming: (isRenaming) => set({ isRenaming }),
  setIsContextMenuOpen: (isContextMenuOpen) => set({ isContextMenuOpen }),
  reset: () => set({ isRenaming: false, isContextMenuOpen: false }),
}))
