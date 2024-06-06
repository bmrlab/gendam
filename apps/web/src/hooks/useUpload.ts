import { useExplorerContext } from '@/Explorer/hooks'
import { useUploadQueueStore } from '@/components/UploadQueue/store'
import { useCallback } from 'react'

export const useUpload = () => {
  const explorer = useExplorerContext()
  const uploadQueueStore = useUploadQueueStore()
  const handleSelectFiles = useCallback(
    (fileFullPaths: string[]) => {
      if (explorer.materializedPath) {
        for (const fileFullPath of fileFullPaths) {
          const name = fileFullPath.split('/').slice(-1).join('')
          uploadQueueStore.enqueue({
            materializedPath: explorer.materializedPath,
            name: name,
            localFullPath: fileFullPath,
          })
        }
      }
    },
    [explorer.materializedPath, uploadQueueStore],
  )
  return {
    handleSelectFiles,
  }
}
