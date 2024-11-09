'use client'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks'
import Icon from '@gendam/ui/icons'
import { Tooltip } from '@gendam/ui/v2/tooltip'
import classNames from 'classnames'
import { useCallback, useMemo } from 'react'
import SearchResultItem from './SearchResultItem'
import { type ItemWithSize } from './SearchResults'

const SearchViewItem: React.FC<ItemWithSize> = ({ data, width, height }) => {
  const explorer = useExplorerContext()
  const quickViewStore = useQuickViewStore()

  const isSelected = useMemo(() => {
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
          'overflow-hidden rounded-lg border-2',
          'flex flex-col items-stretch justify-stretch',
          'transition-colors duration-200 ease-in-out',
          isSelected ? 'border-accent' : 'border-app-line/75',
        )}
        style={{ width: `${width}px`, height: `${height}px` }}
      >
        <div className="group relative w-full flex-1 overflow-hidden">
          <SearchResultItem data={data} />
        </div>
        <Tooltip.Provider delayDuration={200}>
          <Tooltip.Root>
            <Tooltip.Trigger asChild>
              <div className="bg-app-line/75 text-ink/60 flex w-full items-center justify-start gap-1 px-1 py-1 text-xs">
                <Icon.File className="h-4 w-4" />
                <div className="flex-1 origin-left scale-90 truncate">{data.hitReason.reason}</div>
              </div>
            </Tooltip.Trigger>
            <Tooltip.Portal>
              <Tooltip.Content sideOffset={5}>
                <div
                  className="max-h-64 max-w-80 overflow-auto whitespace-pre-line break-words"
                  dangerouslySetInnerHTML={{ __html: data.hitReason.text }}
                ></div>
                <Tooltip.Arrow />
              </Tooltip.Content>
            </Tooltip.Portal>
          </Tooltip.Root>
        </Tooltip.Provider>
      </div>
    </ViewItem>
  )
}

export default SearchViewItem
