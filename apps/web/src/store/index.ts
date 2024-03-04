'use client'

import {createSelectors} from '@/store/createSelectors'

import {create, type StateCreator} from 'zustand'
import {immer} from 'zustand/middleware/immer'
import {AudioDialogSlice, createAudioDialogSlice} from "@/app/video-tasks/store/audio-dialog";

export type ImmerStateCreator<T> = StateCreator<
    T,
    [['zustand/immer', never], never],
    [],
    T
>

type BoundStore = AudioDialogSlice
export const useBoundStoreBase = create<BoundStore>()(
    immer((...a) => ({
        ...createAudioDialogSlice(...a),
    }))
)

export const useBoundStore = createSelectors(useBoundStoreBase)
