import { AssetObject } from '@/lib/bindings'
import { create } from 'zustand'

export type QuickViewItem = {
  name: string
  assetObject: Pick<AssetObject, 'hash' | 'id' | 'mimeType'>
  // video 预览的配置
  video?: {
    currentTime: number  // 初始时间，单位秒
  }
}

interface QuickView {
  show: boolean
  data: QuickViewItem | null
  open: (data: QuickViewItem) => void
  close: () => void
}

export const useQuickViewStore = create<QuickView>((set, get) => ({
  show: false,
  data: null,
  open: (data: QuickViewItem) => set({ show: true, data }),
  close: () => set({ show: false, data: null }),
}))
