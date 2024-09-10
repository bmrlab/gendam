import { useExplorerContext } from '@/Explorer/hooks'
import { ExtractExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useMemo } from 'react'
import { toast } from 'sonner'

export default function VideoExportContextMenu() {
  const explorer = useExplorerContext()
  const exportSegmentMut = rspc.useMutation(['assets.export_video_segment'])

  const [handleExport, valid] = useMemo(() => {
    const selectedItems = Array.from(explorer.selectedItems)

    const validItems = selectedItems
      .map((item) => {
        if (item.type === 'SearchResult' && item.metadata.type === 'video') {
          return item as ExtractExplorerItem<'SearchResult', 'video'>
        } else {
          return void 0
        }
      })
      .filter((v) => !!v)

    if (validItems.length === 0) {
      return [() => void 0, false]
    }

    const exportSegment = async () => {
      const { open } = await import('@tauri-apps/api/dialog')
      const { downloadDir } = await import('@tauri-apps/api/path')
      const selectedDir = await open({
        multiple: false,
        directory: true,
        defaultPath: await downloadDir(),
      })
      if (!selectedDir) {
        return
      }

      for (const { assetObject, metadata, filePaths } of validItems) {
        try {
          await exportSegmentMut.mutateAsync({
            verboseFileName: filePaths[0].name,
            assetObjectId: assetObject.id,
            outputDir: selectedDir as string,
            millisecondsFrom: metadata.startTime,
            millisecondsTo: Math.max(metadata.endTime, metadata.startTime + 1000),
          })
          toast.success('Exported successfully', {
            description: `Exported ${filePaths[0].name} to ${selectedDir}`,
          })
        } catch (err) {
          toast.error('Export failed', {
            description: `Failed to export ${filePaths[0].name} to ${selectedDir}`,
          })
        }
      }
    }

    return [exportSegment, true]
  }, [explorer.selectedItems, exportSegmentMut])

  return (
    <ContextMenu.Item onSelect={handleExport} disabled={!valid}>
      Export
    </ContextMenu.Item>
  )
}
