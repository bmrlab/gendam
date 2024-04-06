import { ExplorerItem } from '@/Explorer/types'
import { create } from 'zustand'

// export type FileItem = {
//   path: string
//   localFullPath: string
// }

interface QuickView {
  show: boolean
  data: ExplorerItem | null
  open: (data: ExplorerItem) => void
  close: () => void
}

export const useQuickViewStore = create<QuickView>((set, get) => ({
  show: false,
  data: null,
  open: (data: ExplorerItem) => set({ show: true, data }),
  close: () => set({ show: false, data: null }),
}))
