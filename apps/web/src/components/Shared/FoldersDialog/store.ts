import { ExplorerItem } from '@/Explorer/types'
import { create } from 'zustand'

interface FoldersDialogState {
  open: boolean
  setOpen: (open: boolean) => void
  confirm: (path: ExplorerItem | null) => void
  setConfirm: (confirm: (path: ExplorerItem | null) => void) => void
}

export const useFoldersDialog = create<FoldersDialogState>((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
  confirm: (path: ExplorerItem | null) => undefined,
  setConfirm: (confirm: (path: ExplorerItem | null) => void) => set({ confirm }),
}))
