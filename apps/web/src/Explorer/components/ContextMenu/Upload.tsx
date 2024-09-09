import { useExplorerContext } from '@/Explorer/hooks'
import { ExtractExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useCallback, useMemo } from 'react'
import { toast } from 'sonner'
import { BaseContextMenuItem } from './types'

function withUploadExplorerItem(BaseComponent: BaseContextMenuItem) {
  return () => {
    const explorer = useExplorerContext()
    const { mutateAsync: uploadToS3 } = rspc.useMutation(['storage.upload_to_s3'])

    type T = ExtractExplorerItem<'FilePath'>
    const selectedFilePathItems: T[] = useMemo(() => {
      return Array.from(explorer.selectedItems).filter((item) => item.type === 'FilePath') as T[]
    }, [explorer.selectedItems])

    const handleUpload = useCallback(async () => {
      let hashes = selectedFilePathItems.map((s) => s.assetObject?.hash).filter((s) => !!s) as string[]
      let materializedPaths = selectedFilePathItems
        .filter((s) => s.filePath.isDir)
        .map((s) => `${s.filePath.materializedPath}${s.filePath.name}/`)
      let payload = {
        hashes,
        materializedPaths,
      }

      if (hashes.length > 0 || materializedPaths.length > 0) {
        try {
          await uploadToS3(payload)
          toast.success('Upload success')
        } catch (error) {
          toast.error('Upload failed')
        }
      }
    }, [selectedFilePathItems])

    return (
      <ContextMenu.Item onSelect={handleUpload} disabled={explorer.selectedItems.size > 1}>
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withUploadExplorerItem(() => <div>Upload</div>)
