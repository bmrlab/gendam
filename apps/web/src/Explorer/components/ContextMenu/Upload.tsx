import { useExplorerContext } from '@/Explorer/hooks'
import { ExtractExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useCallback, useMemo } from 'react'
import { toast } from 'sonner'
import { BaseContextMenuItem } from './types'

function withUploadExplorerItem(BaseComponent: BaseContextMenuItem) {
  return function ContextMenuUpload() {
    const explorer = useExplorerContext()
    const { mutateAsync: uploadToS3 } = rspc.useMutation(['storage.upload_to_s3'])

    type T = ExtractExplorerItem<'FilePathDir' | 'FilePathWithAssetObject'>
    const selectedFilePathItems: T[] = useMemo(() => {
      return Array.from(explorer.selectedItems).filter(
        (item) => item.type === 'FilePathDir' || item.type === 'FilePathWithAssetObject',
      ) as T[]
    }, [explorer.selectedItems])

    const handleUpload = useCallback(async () => {
      const hashes = selectedFilePathItems
        .filter((s) => s.type === 'FilePathWithAssetObject')
        .map((s) => s.assetObject.hash)
        .filter((s) => !!s) as string[]
      const materializedPaths = selectedFilePathItems
        .filter((s) => s.type === 'FilePathDir')
        .map((s) => `${s.filePath.materializedPath}${s.filePath.name}/`)
      const payload = {
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
    }, [selectedFilePathItems, uploadToS3])

    return (
      <ContextMenu.Item onSelect={handleUpload} disabled={explorer.selectedItems.size > 1}>
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withUploadExplorerItem(() => <div>Upload</div>)
