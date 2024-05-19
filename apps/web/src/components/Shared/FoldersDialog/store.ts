import { type FilePath } from '@/lib/bindings'
import { create } from 'zustand'

interface FoldersDialogState {
  open: boolean
  setOpen: (open: boolean) => void
  confirm: (path: FilePath | null) => void
  setConfirm: (confirm: (path: FilePath | null) => void) => void
}

export const useFoldersDialog = create<FoldersDialogState>((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
  confirm: (path: FilePath | null) => undefined,
  setConfirm: (confirm: (path: FilePath | null) => void) => set({ confirm }),
}))
