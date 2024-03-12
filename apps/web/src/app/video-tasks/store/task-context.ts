import { ImmerStateCreator } from '@/store'
import { VideoItem } from '@/app/video-tasks/_components/task-item'

type TaskSelectedState = VideoItem[]

type State = {
  taskSelected: TaskSelectedState
}

type Action = {
  setTaskSelected: (selected: State['taskSelected']) => void
  addTaskSelected: (selected: TaskSelectedState[number] | TaskSelectedState) => void
  removeTaskSelected: (fileHash: string) => void
  clearTaskSelected: () => void
}

export type TaskContextSlice = State & Action

export const createTaskContextSlice: ImmerStateCreator<TaskContextSlice> = (set) => ({
  taskSelected: [],
  setTaskSelected: (taskSelected) => set({ taskSelected }),
  addTaskSelected: (item) =>
    set((state) => {
      const items = Array.isArray(item) ? item : [item]
      const alreadySelectedFileHash = state.taskSelected.map((item) => item.videoFileHash)
      const needAdd = items.filter((item) => !alreadySelectedFileHash.includes(item.videoFileHash))
      state.taskSelected.push(...needAdd)
    }),
  removeTaskSelected: (fileHash) =>
    set((state) => {
      state.taskSelected = state.taskSelected.filter((item) => item.videoFileHash !== fileHash)
    }),
  clearTaskSelected: () => set({ taskSelected: [] }),
})
