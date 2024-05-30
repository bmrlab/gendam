'use client'
import { useExplorerContext } from '@/Explorer/hooks'
import { type ExplorerItem } from '@/Explorer/types'
import { client, rspc } from '@/lib/rspc'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useRouter } from 'next/navigation'
import { forwardRef, useCallback, useMemo } from 'react'
import { toast } from 'sonner'
import { useSearchPageContext, type SearchResultPayload } from './context'

type SearchItemContextMenuProps = {
  data: SearchResultPayload
}

const SearchItemContextMenu = forwardRef<typeof ContextMenu.Content, SearchItemContextMenuProps>(
  function SearchItemContextMenuComponent({ data, ...prpos }, forwardedRef) {
    const explorer = useExplorerContext()
    const router = useRouter()
    const quickViewStore = useQuickViewStore()

    const searchQuery = useSearchPageContext()

    const selectedSearchResultItems = useMemo(() => {
      type T = Extract<ExplorerItem, { type: 'SearchResult' }>
      const filtered = Array.from(explorer.selectedItems).filter((item) => item.type === 'SearchResult') as T[]
      return filtered.map(({ filePath, metadata }) => ({ filePath, metadata } as SearchResultPayload))
    }, [explorer.selectedItems])

    const quickview = useCallback(() => {
      quickViewStore.open({
        name: data.filePath.name,
        assetObject: data.filePath.assetObject!,
        video: {
          currentTime: data.metadata.startTime / 1e3,
        },
      })
    }, [data, quickViewStore])

    const reveal = useCallback(() => {
      router.push(`/explorer?dir=${data.filePath.materializedPath}&id=${data.filePath.id}`)
    }, [data, router])

    const exportSegmentMut = rspc.useMutation(['assets.export_video_segment'])

    const exportSegment = useCallback(async () => {
      const { open, save } = await import('@tauri-apps/api/dialog')
      const { downloadDir } = await import('@tauri-apps/api/path')
      const selectedDir = await open({
        multiple: false,
        directory: true,
        defaultPath: await downloadDir(),
      })
      if (!selectedDir) {
        return
      }
      for (const { filePath, metadata } of selectedSearchResultItems) {
        if (!filePath.assetObjectId) {
          continue
        }
        try {
          await exportSegmentMut.mutateAsync({
            verboseFileName: filePath.name,
            assetObjectId: filePath.assetObjectId,
            outputDir: selectedDir as string,
            millisecondsFrom: metadata.startTime,
            millisecondsTo: Math.max(metadata.endTime, metadata.startTime + 1000),
          })
          toast.success('Exported successfully', {
            description: `Exported ${filePath.name} to ${selectedDir}`
          })
        } catch(err) {
          toast.error('Export failed', {
            description: `Failed to export ${filePath.name} to ${selectedDir}`
          })
        }
      }
    }, [exportSegmentMut, selectedSearchResultItems])

    const recommendFrames = useCallback(async () => {
      if (!data.filePath.assetObject) {
        return
      }
      searchQuery.fetch({
        api: 'search.recommend',
        filePath: data.filePath,
        assetObjectHash: data.filePath.assetObject.hash,
        timestamp: data.metadata.startTime,
      })
    }, [data.filePath, data.metadata, searchQuery])

    return (
      <ContextMenu.Content ref={forwardedRef as any} {...prpos} onClick={(e) => e.stopPropagation()}>
        <ContextMenu.Item
          onSelect={() => quickview()}
          disabled={explorer.selectedItems.size > 1}
        >
          <div>Quick view</div>
        </ContextMenu.Item>
        <ContextMenu.Item
          onSelect={() => reveal()}
          disabled={explorer.selectedItems.size > 1}
        >
          <div>Reveal in explorer</div>
        </ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item
          onSelect={() => recommendFrames()}
          disabled={explorer.selectedItems.size > 1}
        >
          <div>Find similar items</div>
        </ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item
          onSelect={() => exportSegment()}
        >
          <div>Export</div>
        </ContextMenu.Item>
      </ContextMenu.Content>
    )
  },
)

export default SearchItemContextMenu
