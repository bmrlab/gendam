import UploadButton from '@/components/UploadButton'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import { useFileDrop } from '@/hooks/useFileDrop'
import { useClipboardPaste } from '@/hooks/useClipboardPaste'
import { useUploadQueueStore } from '@/components/UploadQueue/store'
import { filterFiles } from '@/components/UploadQueue/utils'
import { toast } from 'sonner'
import { useExplorerContext } from '@/Explorer/hooks'
import { useCallback, useEffect } from 'react'


export const UploadBtn = () => {
  const explorer = useExplorerContext()
  const uploadQueueStore = useUploadQueueStore()

  const { filesDropped } = useFileDrop()
  useEffect(() => {
    if (filesDropped.length > 0) {
      handleSelectFiles(filesDropped)
    }
  }, [filesDropped])

  const { filesPasted } = useClipboardPaste()
  useEffect(() => {
    if (filesPasted.length > 0) {
      handleSelectFiles(filesPasted)
    }
  }, [filesPasted])

  const handleSelectFiles = useCallback(
    (fileFullPaths: string[]) => {
      const { supportedFiles, unsupportedExtensionsSet } = filterFiles(fileFullPaths)
      if (Array.from(unsupportedExtensionsSet).length > 0) {
        toast.error(`Unsupported file types: ${Array.from(unsupportedExtensionsSet).join(',')}`)
      }
      if (explorer.materializedPath && supportedFiles.length > 0) {
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
  return (
    <Button variant="ghost" size="sm" className="h-7 w-7 p-1 transition-none" asChild>
      {/* 加上 asChild 不使用 native button, 因为里面是个 form, native button 可能会触发 form submit */}
      <UploadButton onSelectFiles={handleSelectFiles}>
        <Icon.Upload className="size-4" />
      </UploadButton>
    </Button>
  )
}
