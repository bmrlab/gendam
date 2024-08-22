import { useAudioDialog } from '@/components/TranscriptExport/AudioDialog'
import { useExplorerContext } from '@/Explorer/hooks'
import { ExtractExplorerItem } from '@/Explorer/types'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useMemo } from 'react'

export default function AudioTranscriptContextMenuList() {
  const explorer = useExplorerContext()
  const audioDialog = useAudioDialog()

  type T = ExtractExplorerItem<'FilePath', 'video' | 'audio'>
  const selectedFilePathItems = useMemo(() => {
    return Array.from(explorer.selectedItems).filter(
      (item) =>
        item.type === 'FilePath' &&
        !!item.assetObject &&
        ((item.assetObject.mediaData?.contentType === 'video' && !!item.assetObject.mediaData.audio) ||
          item.assetObject.mediaData?.contentType === 'audio'),
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
