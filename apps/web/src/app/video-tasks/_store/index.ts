'use client'
import { createTaskActionSlice, TaskActionSlice } from '@/app/video-tasks/_store/task-action'
import { createSelectors } from '@/store/createSelectors'
import { create, type StateCreator } from 'zustand'
import { immer } from 'zustand/middleware/immer'
import { AudioDialogSlice, createAudioDialogSlice } from './audio-dialog'
import { createSearchSlice, SearchSlice } from './search'
import { createTaskContextSlice, TaskContextSlice } from './task-context'

export type ImmerStateCreator<T> = StateCreator<T, [['zustand/immer', never], never], [], T>

type BoundStore = AudioDialogSlice & TaskContextSlice & SearchSlice & TaskActionSlice
export const useBoundStoreBase = create<BoundStore>()(
  immer((...a) => ({
    ...createAudioDialogSlice(...a),
    ...createTaskContextSlice(...a),
    ...createSearchSlice(...a),
    ...createTaskActionSlice(...a),
  })),
)

export const useBoundStore = createSelectors(useBoundStoreBase)
