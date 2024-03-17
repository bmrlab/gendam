import { create } from 'zustand'

interface ExplorerState {
  isRenaming: boolean
  isContextMenuOpen: boolean
  setIsRenaming: (isRenaming: boolean) => void
  setIsContextMenuOpen: (isContextMenuOpen: boolean) => void
  reset: () => void
}

export const useExplorerStore = create<ExplorerState>((set) => ({
  isRenaming: false,
  isContextMenuOpen: false,
  setIsRenaming: (isRenaming) => set({ isRenaming }),
  setIsContextMenuOpen: (isContextMenuOpen) => set({ isContextMenuOpen }),
  reset: () => set({ isRenaming: false, isContextMenuOpen: false }),
}))
