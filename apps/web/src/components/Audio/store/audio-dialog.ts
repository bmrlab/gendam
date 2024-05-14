import { BatchExportProps } from '../AudioBatchExport'
import { type StateCreator } from 'zustand'

export enum AudioDialogEnum {
  single,
  batch,
}

export type SingleExportProps = {
  fileHash: string
}

type AudioDialogProps =
  | {
      type: AudioDialogEnum.single
      title: string
      params: SingleExportProps
    }
  | {
      type: AudioDialogEnum.batch
      title: string
      params: BatchExportProps
    }

type State = {
  isOpenAudioDialog: boolean
  audioDialogProps: AudioDialogProps
}

type Action = {
  setIsOpenAudioDialog: (isOpen: State['isOpenAudioDialog']) => void
  setAudioDialogProps: (confirmDialogProps: State['audioDialogProps']) => void
}

export type AudioDialogSlice = State & Action

export const createAudioDialogSlice: StateCreator<AudioDialogSlice, [], [], AudioDialogSlice> = (set) => ({
  isOpenAudioDialog: false,
  audioDialogProps: {
    type: AudioDialogEnum.single,
    title: '',
    params: {
      fileHash: '',
    },
  },
  setIsOpenAudioDialog: (isOpen: boolean) => set(() => ({ isOpenAudioDialog: isOpen })),
  setAudioDialogProps: (audioDialogProps: AudioDialogProps) =>
    set((state) => ({
      audioDialogProps: {
        ...state.audioDialogProps,
        ...audioDialogProps,
      },
    })),
})
