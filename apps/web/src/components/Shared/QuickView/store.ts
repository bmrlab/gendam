import { ExplorerItem, RawFilePath } from '@/Explorer/types'
import { AssetObject } from '@/lib/bindings'
import { create } from 'zustand'

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
