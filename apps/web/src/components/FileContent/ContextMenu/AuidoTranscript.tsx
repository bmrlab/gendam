import { useAudioDialog } from '@/components/TranscriptExport/AudioDialog'
import { useExplorerContext } from '@/Explorer/hooks'
import { ExtractExplorerItem } from '@/Explorer/types'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useMemo } from 'react'

export default function AudioTranscriptContextMenuList() {
  const explorer = useExplorerContext()
  const audioDialog = useAudioDialog()

  type T = ExtractExplorerItem<'FilePathWithAssetObject', 'Video' | 'Audio'>
  const selectedFilePathItems = useMemo(() => {
    return Array.from(explorer.selectedItems).filter(
      (item) =>
        item.type === 'FilePathWithAssetObject' &&
        ((item.assetObject.mediaData?.contentType === 'Video' && !!item.assetObject.mediaData?.audio) ||
          item.assetObject.mediaData?.contentType === 'Audio'),
    ) as T[]
  }, [explorer.selectedItems])

  return (
    <>
      {selectedFilePathItems.length > 0 && (
        <ContextMenu.Item
          onSelect={() => {
            const items = selectedFilePathItems
            return items.length === 1 ? audioDialog.singleExport(items[0]) : audioDialog.batchExport(items)
          }}
        >
          <div>Export transcript</div>
        </ContextMenu.Item>
      )}
    </>
  )
}
