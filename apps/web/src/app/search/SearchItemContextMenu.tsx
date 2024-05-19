'use client'
import { useExplorerContext } from '@/Explorer/hooks'
import { type ExplorerItem } from '@/Explorer/types'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import { type SearchResultPayload } from '@/lib/bindings'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useRouter } from 'next/navigation'
import { forwardRef, useCallback, useMemo } from 'react'

type SearchItemContextMenuProps = {
  data: SearchResultPayload
}

const SearchItemContextMenu = forwardRef<typeof ContextMenu.Content, SearchItemContextMenuProps>(
  function SearchItemContextMenuComponent({ data, ...prpos }, forwardedRef) {
    const explorer = useExplorerContext()
    const router = useRouter()
    const quickViewStore = useQuickViewStore()

    // const selectedSearchResultItems = useMemo(() => {
    //   type T = Extract<ExplorerItem, { type: 'SearchResult' }>
    //   const filtered = Array.from(explorer.selectedItems).filter((item) => item.type === 'SearchResult') as T[]
    //   return filtered.map(({ filePath, metadata }) => ({ filePath, metadata } as SearchResultPayload))
    // }, [explorer.selectedItems])

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
      </ContextMenu.Content>
    )
  },
)

export default SearchItemContextMenu
