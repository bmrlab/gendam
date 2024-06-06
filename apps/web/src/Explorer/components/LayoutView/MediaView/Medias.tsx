'use client'
import { useExplorerDroppableContext } from '@/Explorer/components/Draggable/ExplorerDroppable'
import FileThumb from '@/Explorer/components/View/FileThumb'
import RenamableItemText from '@/Explorer/components/View/RenamableItemText'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { uniqueId } from '@/Explorer/types'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import classNames from 'classnames'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { type WithFilePathExplorerItem } from './index'

type ItemWithSize = {
  data: WithFilePathExplorerItem
  width: number
  height: number
}

const DroppableInner: React.FC<ItemWithSize> = ({ data, width, height }) => {
  // className 和 props 没有用到
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const { isDroppable } = useExplorerDroppableContext()
  const highlight = useMemo(() => {
    return explorer.isItemSelected(data) || isDroppable
  }, [data, explorer, isDroppable])

  const [name1, name2] = useMemo(() => {
    if (/\.[^.]{1,5}$/i.test(data.filePath.name)) {
      return [data.filePath.name.slice(0, -8), data.filePath.name.slice(-8)]
    } else {
      return [data.filePath.name.slice(0, -4), data.filePath.name.slice(-4)]
    }
  }, [data.filePath.name])

  return (
    <div style={{ width: `${width}px` }}>
      <div
        className={classNames('mb-1', highlight ? 'bg-app-hover' : null)}
        style={{ width: `100%`, height: `${height}px` }}
      >
        <FileThumb data={data} className="h-full w-full" />
      </div>
      {explorer.isItemSelected(data) && explorerStore.isRenaming ? (
        <div>
          <RenamableItemText data={data} className="text-center" />
        </div>
      ) : (
        <div
          className={classNames(
            'text-ink flex items-center justify-center overflow-hidden rounded-lg px-2 py-1 text-xs',
            highlight ? 'bg-accent text-white' : null,
          )}
        >
          <div className="truncate whitespace-pre">{name1}</div>
          <div className="whitespace-pre">{name2}</div>
        </div>
      )}
    </div>
  )
}

const MediaItem: React.FC<
  ItemWithSize & {
    onSelect: (e: React.MouseEvent, data: WithFilePathExplorerItem) => void
  }
> = ({ data, onSelect, ...props }) => {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const quickViewStore = useQuickViewStore()

  const handleDoubleClick = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      explorer.resetSelectedItems()
      explorerStore.reset()
      if (data.filePath.assetObject) {
        const { name, assetObject } = data.filePath
        quickViewStore.open({ name, assetObject })
      }
    },
    [data, explorer, explorerStore, quickViewStore],
  )

  return (
    <ViewItem data={data} onClick={(e) => onSelect(e, data)} onDoubleClick={handleDoubleClick}>
      <DroppableInner data={data} {...props} />
    </ViewItem>
  )
}

export default function Medias({ items }: { items: WithFilePathExplorerItem[] }) {
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

  const itemsWithSize = useMemo<ItemWithSize[]>(() => {
    if (!containerWidth) {
      return []
    }
    let itemsTotalWidth = 0
    let itemsWithSize: ItemWithSize[] = []
    let queue: ItemWithSize[] = []
    for (let data of items) {
      const mediaWidth = data.filePath.assetObject?.mediaData?.width || 100 // px
      const mediaHeight = data.filePath.assetObject?.mediaData?.height || 100 // px
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
    (e: React.MouseEvent, data: WithFilePathExplorerItem) => {
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
        <MediaItem
          key={uniqueId(data)}
          data={data}
          onSelect={onSelect}
          width={Math.floor(width)}
          height={Math.floor(height)}
        />
      ))}
    </div>
  )
}
