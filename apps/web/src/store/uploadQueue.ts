import { create } from 'zustand'

export type FileItem = {
  path: string
  localFullPath: string
}

interface UploadQueue {
  completed: FileItem[]
  failed: FileItem[]
  queue: FileItem[]
  uploading: FileItem | null
  nextUploading: () => FileItem | null
  completeUploading: () => void
  failedUploading: () => void
  enqueue: (item: FileItem) => void
}

export const useUploadQueueStore = create<UploadQueue>((set, get) => ({
  completed: [],
  failed: [],
  queue: [],
  uploading: null,
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
  completeUploading: () => {
    set((state) => {
      if (state.uploading) {
        return {
          completed: [state.uploading, ...state.completed],
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
          failed: [state.uploading, ...state.completed],
          uploading: null,
        }
      }
      return {}
    })
  },
  enqueue: (item) => {
    set((state) => ({
      queue: [...state.queue, item],
    }))
  },
}))
