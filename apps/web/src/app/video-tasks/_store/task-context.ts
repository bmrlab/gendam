import { ImmerStateCreator } from '.'
import type { VideoWithTasksResult } from '@/lib/bindings'

type VideoSelectedState = VideoWithTasksResult[]

type State = {
  videoSelected: VideoSelectedState
}

type Action = {
  setVideoSelected: (selected: State['videoSelected']) => void
  addVideoSelected: (selected: VideoSelectedState[number] | VideoSelectedState) => void
  removeVideoSelected: (assetObjectId: number) => void
  clearVideoSelected: () => void
}

export type TaskContextSlice = State & Action

export const createTaskContextSlice: ImmerStateCreator<TaskContextSlice> = (set) => ({
  videoSelected: [],
  setVideoSelected: (videoSelected) => set({ videoSelected }),
  addVideoSelected: (item) =>
    set((state) => {
      const items = Array.isArray(item) ? item : [item]
      const alreadySelectedId = state.videoSelected.map((item) => item.assetObjectId)
      const needAdd = items.filter((item) => !alreadySelectedId.includes(item.assetObjectId))
      state.videoSelected.push(...needAdd)
    }),
  removeVideoSelected: (assetObjectId) =>
    set((state) => {
      state.videoSelected = state.videoSelected.filter((item) => item.assetObjectId !== assetObjectId)
    }),
  clearVideoSelected: () => set({ videoSelected: [] }),
})
