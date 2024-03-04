import {type StateCreator} from 'zustand'

type AudioDialogProps = {}

type State = {
    isOpenAudioDialog: boolean
    audioDialogProps: AudioDialogProps
}

type Action = {
    setIsOpenAudioDialog: (isOpen: State['isOpenAudioDialog']) => void
    setAudioDialogProps: (
        confirmDialogProps: State['audioDialogProps']
    ) => void
}

export type AudioDialogSlice = State & Action

export const createAudioDialogSlice: StateCreator<
    AudioDialogSlice,
    [],
    [],
    AudioDialogSlice
> = (set) => ({
    isOpenAudioDialog: false,
    audioDialogProps: {},
    setIsOpenAudioDialog: (isOpen: boolean) =>
        set(() => ({isOpenAudioDialog: isOpen})),
    setAudioDialogProps: (audioDialogProps: AudioDialogProps) =>
        set((state) => ({
            audioDialogProps: {
                ...state.audioDialogProps,
                ...audioDialogProps,
            },
        })),
})
