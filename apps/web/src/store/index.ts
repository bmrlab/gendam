'use client'

import { createSelectors } from '@/store/createSelectors'

import { AudioDialogSlice, createAudioDialogSlice } from '@/app/video-tasks/store/audio-dialog'
import { TaskContextSlice, createTaskContextSlice } from '@/app/video-tasks/store/task-context'
import { create, type StateCreator } from 'zustand'
import { immer } from 'zustand/middleware/immer'

export type ImmerStateCreator<T> = StateCreator<T, [['zustand/immer', never], never], [], T>

type BoundStore = AudioDialogSlice & TaskContextSlice
export const useBoundStoreBase = create<BoundStore>()(
  immer((...a) => ({
    ...createAudioDialogSlice(...a),
    ...createTaskContextSlice(...a),
  })),
)

export const useBoundStore = createSelectors(useBoundStoreBase)
