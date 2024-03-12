import { type StateCreator } from 'zustand'

type State = {
  searchKey: string
}

type Action = {
  setSearchKey: (searchKey: State['searchKey']) => void
}

export type SearchSlice = State & Action

export const createSearchSlice: StateCreator<SearchSlice, [], [], SearchSlice> = (set) => ({
  searchKey: '',
  setSearchKey: (searchKey: string) => set(() => ({ searchKey: searchKey })),
})
