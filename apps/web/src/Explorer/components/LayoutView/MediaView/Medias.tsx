'use client'
import ExplorerDraggable from '@/Explorer/components/Draggable/ExplorerDraggable'
import ExplorerDroppable, { useExplorerDroppableContext } from '@/Explorer/components/Draggable/ExplorerDroppable'
import FileThumb from '@/Explorer/components/View/FileThumb'
import RenamableItemText from '@/Explorer/components/View/RenamableItemText'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { SELECTABLE_TARGETS_IDS } from '@/Explorer/constant'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import classNames from 'classnames'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

type ItemsWithSize = {
  data: ExplorerItem
  width: number
  height: number
}

const DroppableInner: React.FC<ItemsWithSize> = ({ data, width, height }) => {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  width = Math.floor(width)
  height = Math.floor(height)

  const { isDroppable } = useExplorerDroppableContext()
  const highlight = useMemo(() => {
    return explorer.isItemSelected(data) || isDroppable
  }, [data, explorer, isDroppable])

  return (
    <>
      <div
        className={classNames('mb-1 overflow-hidden', highlight ? 'bg-app-hover' : null)}
        style={{ width: `${width}px`, height: `${height}px` }}
      >
        <FileThumb data={data} className="h-full w-full" />
      </div>
      {explorer.isItemSelected(data) && explorerStore.isRenaming ? (
        <div style={{ width: `${width}px` }}>
          <RenamableItemText data={data} className="text-center" />
        </div>
      ) : (
        <div
          style={{ width: `${width}px` }}
          className={classNames('text-ink rounded p-1', highlight ? 'bg-accent text-white' : null)}
        >
          <div className="line-clamp-2 max-h-[2.8em] break-all text-center text-xs leading-[1.4em]">{data.name}</div>
        </div>
      )}
    </>
  )
}

const MediaItem: React.FC<
  ItemsWithSize & {
    onSelect: (e: React.MouseEvent, data: ExplorerItem) => void
  }
> = ({ data, width, height, onSelect }) => {
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
      id={SELECTABLE_TARGETS_IDS[2]}
      itemID={data.id.toString()}
      data-component-hint="ViewItem(MediaView,Media)"
      onClick={(e) => {
        e.stopPropagation()
        onSelect(e, data)
      }}
      onDoubleClick={handleDoubleClick}
    >
      <ViewItem data={data}>
        <ExplorerDroppable droppable={{ data: data }}>
          <ExplorerDraggable draggable={{ data: data }}>
            <DroppableInner data={data} width={width} height={height} />
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

  const ref = useRef<HTMLDivElement>(null)
  const gap = 10
  const padding = 30 // container 左右 padding
  const [containerWidth, setContainerWidth] = useState<number>(0)

  useEffect(() => {
    const $el = ref.current
    if (!$el) {
      return
    }
    // ref.current 必须在 useEffect 里面用, 不然还没有 mount, 它还是 undefined
    const containerWidth = Math.max(0, ($el.clientWidth || 0) - padding * 2)
    setContainerWidth(containerWidth)
    const resizeObserver = new ResizeObserver((entries) => {
      for (let entry of entries) {
        if (entry.target === $el) {
          const containerWidth = Math.max(0, ($el.clientWidth || 0) - padding * 2)
          setContainerWidth(containerWidth)
        }
      }
    })
    resizeObserver.observe($el)
    return () => {
      resizeObserver.unobserve($el)
    }
  }, [])

  const itemsWithSize = useMemo<ItemsWithSize[]>(() => {
    if (!containerWidth) {
      return []
    }
    let itemsTotalWidth = 0
    let itemsWithSize: ItemsWithSize[] = []
    let queue: ItemsWithSize[] = []
    for (let data of items) {
      const mediaWidth = data.assetObject?.mediaData?.width || 100 // px
      const mediaHeight = data.assetObject?.mediaData?.height || 100 // px
      /* 高度 100px, 宽度 100 ~ 300 px */
      const height = 150
      const width = Math.min(Math.min(450, containerWidth), Math.max(100, (height * mediaWidth) / mediaHeight))
      const maxTotalWidth = containerWidth - gap * (queue.length - 1)
      if (itemsTotalWidth + width > maxTotalWidth) {
        const scale = maxTotalWidth / itemsTotalWidth
        queue.forEach((item) => {
          item.width *= scale
          item.height *= scale
        })
        itemsWithSize = itemsWithSize.concat(queue)
        itemsTotalWidth = 0
        queue.length = 0
      }
      itemsTotalWidth += width
      queue.push({ data, width, height })
    }
    itemsWithSize = [...itemsWithSize, ...queue]
    return itemsWithSize
  }, [containerWidth, items])

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

  useEffect(() => {
    // 右键点开以后重置 shift 批量选择的状态，因为右键菜单打开的时候会选中对应的条目
    if (explorerStore.isContextMenuOpen && lastSelectIndex) {
      setLastSelectedIndex(-1)
    }
  }, [explorerStore.isContextMenuOpen, lastSelectIndex])

  return (
    <div
      ref={ref}
      className="flex w-full flex-wrap content-start items-start justify-start"
      style={{ columnGap: `${gap}px`, rowGap: '30px', padding: `${padding}px` }}
    >
      {itemsWithSize.map(({ data, width, height }) => (
        <MediaItem key={data.id} data={data} width={width} height={height} onSelect={onSelect} />
      ))}
    </div>
  )
}
