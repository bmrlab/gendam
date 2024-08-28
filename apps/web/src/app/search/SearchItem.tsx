'use client'

import SearchResultItem from '@/components/FileContent/Search'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks'
import classNames from 'classnames'
import { useCallback, useMemo } from 'react'
import { type ItemWithSize } from './SearchResults'

const SearchItem: React.FC<ItemWithSize> = ({ data, width, height }) => {
  const explorer = useExplorerContext()
  const quickViewStore = useQuickViewStore()

  const highlight = useMemo(() => {
    return explorer.isItemSelected(data)
  }, [data, explorer])

  const quickview = useCallback(() => {
    quickViewStore.open(data)
  }, [quickViewStore, data])

  const onSelect = useCallback(
    (e: React.MouseEvent) => {
      // 按住 cmd 键多选
      if (e.metaKey) {
        if (explorer.isItemSelected(data)) {
          explorer.removeSelectedItem(data)
        } else {
          explorer.addSelectedItem(data)
        }
      } else {
        explorer.resetSelectedItems([data])
      }
      // explorerStore.reset()
    },
    [explorer, data],
  )

  return (
    <ViewItem data={data} onClick={onSelect} onDoubleClick={() => quickview()} isDraggable={false}>
      <div
        className={classNames(
          'group relative overflow-hidden rounded-xl border-4',
          // 'transition-all duration-200 ease-in-out',
          highlight ? 'border-accent' : 'border-app-line/75',
        )}
        style={{ width: `${width}px`, height: `${height}px` }}
      >
        <SearchResultItem data={data} />
      </div>
    </ViewItem>
  )
}

export default SearchItem
