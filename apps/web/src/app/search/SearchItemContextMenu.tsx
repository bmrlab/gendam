'use client'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import { useExplorerContext } from '@/Explorer/hooks'
import { ExtractExplorerItem, type ExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useRouter } from 'next/navigation'
import { forwardRef, useCallback, useMemo } from 'react'
import { toast } from 'sonner'
import { useSearchPageContext } from './context'

type SearchItemContextMenuProps = {
  data: ExtractExplorerItem<'SearchResult'>
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
      return filtered.map(
        ({ filePaths, assetObject, metadata }) =>
          ({ filePaths, assetObject, metadata }) as ExtractExplorerItem<'SearchResult'>,
      )
    }, [explorer.selectedItems])

    const quickview = useCallback(() => {
      quickViewStore.open(data)
    }, [data, quickViewStore])

    const reveal = useCallback(() => {
      // FIXME this need to be optimized
      router.push(`/explorer?dir=${data.filePaths[0].materializedPath}&id=${data.filePaths[0].id}`)
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
      for (const { filePaths, assetObject, metadata } of selectedSearchResultItems) {
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
    }, [exportSegmentMut, selectedSearchResultItems])

    const recommendFrames = useCallback(async () => {
      searchQuery.fetch({
        api: 'search.recommend',
        filePath: data.filePaths[0],
        assetObjectHash: data.assetObject.hash,
        timestamp: data.metadata.startTime,
      })
    }, [data.filePaths, data.metadata, searchQuery])

    return (
      <ContextMenu.Content ref={forwardedRef as any} {...prpos} onClick={(e) => e.stopPropagation()}>
        <ContextMenu.Item onSelect={() => quickview()} disabled={explorer.selectedItems.size > 1}>
          <div>Quick view</div>
        </ContextMenu.Item>
        <ContextMenu.Item onSelect={() => reveal()} disabled={explorer.selectedItems.size > 1}>
          <div>Reveal in explorer</div>
        </ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item onSelect={() => recommendFrames()} disabled={explorer.selectedItems.size > 1}>
          <div>Find similar items</div>
        </ContextMenu.Item>
        <ContextMenu.Separator />
        <ContextMenu.Item onSelect={() => exportSegment()}>
          <div>Export</div>
        </ContextMenu.Item>
      </ContextMenu.Content>
    )
  },
)

export default SearchItemContextMenu
