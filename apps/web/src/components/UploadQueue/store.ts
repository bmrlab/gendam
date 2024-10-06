// TODO: move this to Shared Folder
import { SUPPORTED_CONTENT_TYPES } from '@/constants'
import { type FilePath } from '@/lib/bindings'
import { toast } from 'sonner'
import { create } from 'zustand'

export type UploadQueuePayload = {
  materializedPath: string
  //
  name: string
} & (
  | {
      dataType: 'path'
      payload: string
    }
  | {
      dataType: 'file'
      payload: File
    }
)

interface UploadQueue {
  inProcess: FilePath[]
  completed: (UploadQueuePayload & FilePath)[]
  failed: UploadQueuePayload[]
  queue: UploadQueuePayload[]
  uploading: UploadQueuePayload | null
  setInProcessItems: (items: FilePath[]) => void
  nextUploading: () => UploadQueuePayload | null
  completeUploading: (filePathData: FilePath) => void
  failedUploading: () => void
  retryFailed: (item: UploadQueuePayload) => void
  enqueue: (item: UploadQueuePayload) => void
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
  completeUploading: (filePathData: FilePath) => {
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
    const extension = item.name.split('.').pop()?.toLowerCase() ?? ''
    if (!SUPPORTED_CONTENT_TYPES.has(extension)) {
      toast.error(`Unsupported file type: ${extension} for ${item.name}`)
      return
    }
    set((state) => ({
      queue: [...state.queue, item],
    }))
  },
  clear: () => {
    set({
      completed: [],
      failed: [],
      queue: [],
      // uploading: null,
    })
  },
}))
