import {type StateCreator} from 'zustand'

type State = {
    taskListRefetch: () => Promise<unknown>
}

type Action = {
    setTaskListRefetch: (searchKey: State['taskListRefetch']) => void
}

export type TaskActionSlice = State & Action

export const createTaskActionSlice: StateCreator<TaskActionSlice, [], [], TaskActionSlice> = (set) => ({
    taskListRefetch: () => Promise.resolve(),
    setTaskListRefetch: (taskListRefetch: State['taskListRefetch']) => set(() => ({taskListRefetch})),
})
