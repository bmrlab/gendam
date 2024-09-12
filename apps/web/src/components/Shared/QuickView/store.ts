import { type ExtractExplorerItem } from '@/Explorer/types'
import { create } from 'zustand'

interface QuickView {
  show: boolean
  data: ExtractExplorerItem<'FilePathWithAssetObject' | 'SearchResult'> | null
  open: (data: ExtractExplorerItem<'FilePathWithAssetObject' | 'SearchResult'>) => void
  close: () => void
}

export const useQuickViewStore = create<QuickView>((set, get) => ({
  show: false,
  data: null,
  open: (data: ExtractExplorerItem<'FilePathWithAssetObject' | 'SearchResult'>) => set({ show: true, data }),
  close: () => set({ show: false, data: null }),
}))
