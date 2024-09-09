import { useExplorerContext } from '@/Explorer/hooks'
import { matchRetrievalResult, matchSearchResult } from '@/Explorer/pattern'
import { useSearchPageContext } from '@/app/search/context'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useMemo } from 'react'
import { match, P } from 'ts-pattern'
import { BaseContextMenuItem } from './types'

function withFindSimilarExplorerItem(BaseComponent: BaseContextMenuItem) {
  return () => {
    const explorer = useExplorerContext()
    const searchQuery = useSearchPageContext()

    const [handleFindSimilar, valid] = useMemo(() => {
      const data = Array.from(explorer.selectedItems).at(0)

      if (!data) return [() => void 0, false]

      return match(data)
        .with(P.union(matchSearchResult('video'), matchRetrievalResult('video')), (item) => {
          return [
            () => {
              searchQuery.fetch({
                api: 'search.recommend',
                assetObjectHash: item.assetObject.hash,
                timestamp: item.metadata.startTime,
                filePath: 'filePaths' in item ? item.filePaths[0] : void 0,
              })
            },
            // FIXME disable find similar items for now, because search.recommend is not implemented for now
            // true,
            false,
          ] as const
        })
        .otherwise(() => {
          return [() => void 0, false]
        })
    }, [explorer.selectedItems, searchQuery])

    return (
      <ContextMenu.Item onSelect={handleFindSimilar} disabled={explorer.selectedItems.size > 1 || !valid}>
        <BaseComponent />
      </ContextMenu.Item>
    )
  }
}

export default withFindSimilarExplorerItem(() => <div>Find similar items</div>)
