'use client'
import ExplorerDraggable from '@/Explorer/components/Draggable/ExplorerDraggable'
import ExplorerDroppable, { useExplorerDroppableContext } from '@/Explorer/components/Draggable/ExplorerDroppable'
import FileThumb from '@/Explorer/components/View/FileThumb'
import RenamableItemText from '@/Explorer/components/View/RenamableItemText'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import classNames from 'classnames'
import { useCallback, useMemo, useState } from 'react'

const DroppableInner: React.FC<{ data: ExplorerItem }> = ({ data }) => {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const { isDroppable } = useExplorerDroppableContext()
  const highlight = useMemo(() => {
    return explorer.isItemSelected(data) || isDroppable
  }, [data, explorer, isDroppable])

  return (
    <>
      <div className={classNames('mb-1 h-28 w-28 rounded-lg p-2', highlight ? 'bg-app-hover' : null)}>
        <FileThumb data={data} className="h-full w-full" />
      </div>
      {explorer.isItemSelected(data) && explorerStore.isRenaming ? (
        <div className="w-28">
          <RenamableItemText data={data} className="text-center" />
        </div>
      ) : (
        <div className={classNames('text-ink w-28 rounded-lg p-1', highlight ? 'bg-accent text-white' : null)}>
          <div className="line-clamp-2 max-h-[2.8em] break-all text-center text-xs leading-[1.4em]">{data.name}</div>
        </div>
      )}
    </>
  )
}

const MediaItem: React.FC<{
  data: ExplorerItem
  onSelect: (e: React.MouseEvent, data: ExplorerItem) => void
}> = ({ data, onSelect }) => {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const quickViewStore = useQuickViewStore()

  const handleDoubleClick = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      explorer.resetSelectedItems()
      explorerStore.reset()
      if (data.assetObject) {
        const { name, assetObject } = data
        quickViewStore.open({ name, assetObject })
      }
    },
    [data, explorer, explorerStore, quickViewStore],
  )

  return (
    <div
      className="flex cursor-default select-none flex-col items-center justify-start"
      onClick={(e) => {
        e.stopPropagation()
        onSelect(e, data)
      }}
      onDoubleClick={handleDoubleClick}
    >
      <ViewItem data={data}>
        <ExplorerDroppable droppable={{ data: data }}>
          <ExplorerDraggable draggable={{ data: data }}>
            <DroppableInner data={data} />
          </ExplorerDraggable>
        </ExplorerDroppable>
      </ViewItem>
    </div>
  )
}

export default function Medias({ items }: { items: ExplorerItem[] }) {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const [lastSelectIndex, setLastSelectedIndex] = useState<number>(-1)

  const onSelect = useCallback(
    (e: React.MouseEvent, data: ExplorerItem) => {
      // 只处理 medias 的选择
      const selectIndex = items.indexOf(data)
      if (e.metaKey) {
        if (explorer.isItemSelected(data)) {
          explorer.removeSelectedItem(data)
        } else {
          explorer.addSelectedItem(data)
        }
        setLastSelectedIndex(selectIndex)
      } else if (e.shiftKey) {
        // TODO: 这里要改一下，explorer.selectedItems 也可能包含了文件夹的，比如通过右键选择的
        if (explorer.selectedItems.size > 0 && lastSelectIndex >= 0) {
          const start = Math.min(lastSelectIndex, selectIndex)
          const end = Math.max(lastSelectIndex, selectIndex)
          explorer.resetSelectedItems(items.slice(start, end + 1))
        }
      } else {
        explorer.resetSelectedItems([data])
        setLastSelectedIndex(selectIndex)
      }
      explorerStore.reset()
    },
    [explorer, explorerStore, items, lastSelectIndex],
  )

  return (
    <div className="flex flex-wrap content-start items-start justify-start gap-6 p-8">
      {items.map((item) => (
        <MediaItem key={item.id} data={item} onSelect={onSelect} />
      ))}
    </div>
  )
}
