'use client'
import { useUploadQueueStore, FileItem } from '@/store/uploadQueue'
import { client } from '@/lib/rspc'
import { useCallback, useEffect, useState } from 'react'
import { useExplorerContext } from '@/Explorer/hooks'

export default function UploadQueue() {
  const uploadQueueStore = useUploadQueueStore()
  const explorer = useExplorerContext()

  return (
    <div className="fixed right-4 bottom-4 bg-white border border-neutral-100 shadow-md rounded-md">
      <div className="w-80">
        {uploadQueueStore.queue.map((file, index) => (
          <div key={index} className="px-4 py-2">
            <div className="text-xs overflow-hidden">{file.localFullPath}</div>
          </div>
        ))}
      </div>
    </div>
  )
}
