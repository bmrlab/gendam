// TODO: move this to Shared Folder

import { create } from 'zustand'
import { ExplorerItem } from '@/Explorer/types'

export type FileItem = {
  materializedPath: string
  name: string
  localFullPath: string
}

interface UploadQueue {
  inProcess: ExplorerItem[]
  completed: (FileItem & ExplorerItem)[]
  failed: FileItem[]
  queue: FileItem[]
  uploading: FileItem | null
  setInProcessItems: (items: ExplorerItem[]) => void
  nextUploading: () => FileItem | null
  completeUploading: (filePathData: ExplorerItem) => void
  failedUploading: () => void
  retryFailed: (item: FileItem) => void
  enqueue: (item: FileItem) => void
  clear: () => void
}

export const useUploadQueueStore = create<UploadQueue>((set, get) => ({
  inProcess: [],
  completed: [],
  failed: [],
  queue: [],
  uploading: null,
  setInProcessItems: (items) => {
    set({ inProcess: items })
  },
  nextUploading: () => {
    const { queue, uploading } = get()
    if (uploading || queue.length === 0) {
      return null
    }
    const [newUploading, ...newQueue] = queue
    set({
      uploading: newUploading,
      queue: newQueue,
    })
    return newUploading
  },
  completeUploading: (filePathData: ExplorerItem) => {
    set((state) => {
      if (state.uploading) {
        // should check if state.uploading is same as filePathData (but name may be different ...)
        const newItem = {
          ...state.uploading,
          ...filePathData,
        }
        return {
          completed: [newItem, ...state.completed],
          uploading: null,
        }
      }
      return {}
    })
  },
  failedUploading: () => {
    set((state) => {
      if (state.uploading) {
        return {
          failed: [state.uploading, ...state.failed],
          uploading: null,
        }
      }
      return {}
    })
  },
  retryFailed: (item) => {
    set((state) => ({
      failed: state.failed.filter((failedItem) => failedItem !== item),
      queue: [item, ...state.queue],
    }))
  },
  enqueue: (item) => {
    set((state) => ({
      queue: [...state.queue, item],
    }))
  },
  clear: () => {
    set({
      completed: [],
      failed: [],
      queue: [],
      uploading: null,
    })
  },
}))
