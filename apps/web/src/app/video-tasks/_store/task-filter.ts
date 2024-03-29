import { Filter } from '@/lib/bindings'
import { type StateCreator } from 'zustand'

type State = {
  taskFilter: Filter
}

type Action = {
  setTaskFilter: (taskFilter: State['taskFilter']) => void
}

export type TaskFilterSlice = State & Action

export const createTaskFilterSlice: StateCreator<TaskFilterSlice, [], [], TaskFilterSlice> = (set) => ({
  taskFilter: 'excludeCompleted',
  setTaskFilter: (taskFilter: State['taskFilter']) => set(() => ({ taskFilter })),
})
