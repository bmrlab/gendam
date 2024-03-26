import { create } from 'zustand'

export type FileItem = {
  path: string
  localFullPath: string
}

interface UploadQueue {
  queue: FileItem[]
  uploading: boolean
  setUploading: (uploading: boolean) => void
  dequeue: () => void
  enqueue: (item: FileItem) => void
  peek: () => FileItem | null
}

export const useUploadQueueStore = create<UploadQueue>((set, get) => ({
  queue: [],
  uploading: false,
  setUploading: (uploading) => {
    set({ uploading })
  },
  enqueue: (item) => {
    set((state) => ({
      queue: [...state.queue, item],
    }))
  },
  dequeue: () => {
    set((state) => {
      const [, ...newQueue] = state.queue
      return { queue: newQueue }
    })
  },
  peek: () => {
    const { queue } = get()
    return queue.length > 0 ? queue[0] : null
  },
}))
