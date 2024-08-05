import { PickAssetObject } from '@/components/FileThumb'
import { ContentMetadataWithType, FilePath } from '@/lib/bindings'
import { AssetObjectType } from '@/lib/library'
import { create } from 'zustand'

export type PreviewParams =
  | {
      contentType: Extract<AssetObjectType, 'video'>
      currentTime: number // 初始时间，单位秒
    }
  | {
      contentType: Extract<AssetObjectType, 'audio'>
      currentTime: number // 初始时间，单位秒
    }

export type QuickViewItem = {
  name: string
  assetObject: NonNullable<FilePath['assetObject']>
  params?: PreviewParams
}

export type PickQuickViewItem<T extends ContentMetadataWithType['contentType']> = {
  name: string
  assetObject: PickAssetObject<T>
  params?: Extract<PreviewParams, { contentType: T }>
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
