'use client'
import { useExplorerDroppableContext } from '@/Explorer/components/Draggable/ExplorerDroppable'
import RenamableItemText from '@/Explorer/components/View/RenamableItemText'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { ExtractExplorerItem, uniqueId } from '@/Explorer/types'
import useDebouncedCallback from '@/hooks/useDebouncedCallback'
// import { useCurrentLibrary } from '@/lib/library'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import { useCurrentLibrary } from '@/lib/library'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { HTMLAttributes, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import ThumbItem from '../View/ThumbItem'
// import styles from './GridView.module.css'

const DroppableInner: React.FC<
  {
    data: ExtractExplorerItem<'FilePathDir' | 'FilePathWithAssetObject' | 'SearchResult'>
  } & HTMLAttributes<HTMLDivElement>
> = ({ data, ...props }) => {
  // const currentLibrary = useCurrentLibrary()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const { isDroppable } = useExplorerDroppableContext()
  const highlight = useMemo(() => {
    return explorer.isItemSelected(data) || isDroppable
  }, [data, explorer, isDroppable])

  const filePath = useMemo(() => {
    if ('filePath' in data) return data.filePath
    return data.filePaths.at(0)
  }, [data])

  const [name1, name2] = useMemo(() => {
    if (!filePath) return ['', '']

    if (/\.[^.]{1,5}$/i.test(filePath.name)) {
      return [filePath.name.slice(0, -8), filePath.name.slice(-8)]
    } else {
      return [filePath.name.slice(0, -4), filePath.name.slice(-4)]
    }
  }, [filePath])

  return (
    <div {...props}>
      <div className={classNames('mb-1 h-28 w-full rounded-lg p-2', highlight ? 'bg-app-hover' : null)}>
        <ThumbItem data={data} className="h-full w-full" variant="grid" />
      </div>
      {explorer.isItemSelected(data) &&
      explorerStore.isRenaming &&
      (data.type === 'FilePathDir' || data.type === 'FilePathWithAssetObject') ? (
        <div>
          <RenamableItemText data={data} onClose={() => explorerStore.setIsRenaming(false)} className="text-center" />
        </div>
      ) : (
        <div
          className={classNames(
            'text-ink flex items-center justify-center overflow-hidden rounded-lg px-2 py-1 text-xs',
            highlight ? 'bg-accent text-white' : null,
          )}
        >
          {/* <div className="line-clamp-2 max-h-[2.8em] break-all text-center leading-[1.4em]">{data.name}</div> */}
          <div className="truncate whitespace-pre">{name1}</div>
          <div className="whitespace-pre">{name2}</div>
        </div>
      )}
    </div>
  )
}

const GridItem: React.FC<
  {
    data: ExtractExplorerItem<'FilePathDir' | 'FilePathWithAssetObject' | 'SearchResult'>
    onSelect: (
      e: React.MouseEvent,
      data: ExtractExplorerItem<'FilePathDir' | 'FilePathWithAssetObject' | 'SearchResult'>,
    ) => void
  } & Omit<HTMLAttributes<HTMLDivElement>, 'onSelect'>
> = ({ data, onSelect, ...props }) => {
  const router = useRouter()
  const currentLibrary = useCurrentLibrary()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const quickViewStore = useQuickViewStore()

  const handleDoubleClick = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      // e.stopPropagation()
      explorer.resetSelectedItems()
      explorerStore.reset()
      if (data.type === 'FilePathDir') {
        const newPath = data.filePath.materializedPath + data.filePath.name + '/'
        router.push('/explorer?dir=' + newPath)
      } else if (data.type === 'FilePathWithAssetObject') {
        quickViewStore.open(data)
      }
    },
    [data, explorer, router, explorerStore, quickViewStore],
  )

  return (
    <ViewItem data={data} onClick={(e) => onSelect(e, data)} onDoubleClick={handleDoubleClick} isDraggable={true}>
      <DroppableInner data={data} {...props} />
    </ViewItem>
  )
}

export default function GridView({
  items,
}: {
  items: ExtractExplorerItem<'FilePathDir' | 'FilePathWithAssetObject' | 'SearchResult'>[]
}) {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const [lastSelectIndex, setLastSelectedIndex] = useState<number>(-1)

  const ref = useRef<HTMLDivElement>(null)
  const gap = 10
  const padding = 30 // container 左右 padding
  const [containerWidth, setContainerWidth] = useState<number>(0)

  const debounceSetContainerWidth = useDebouncedCallback((containerWidth: number) => {
    setContainerWidth(containerWidth)
  }, 100)

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
          debounceSetContainerWidth(containerWidth)
          break
        }
      }
    })
    resizeObserver.observe($el)
    return () => {
      resizeObserver.unobserve($el)
    }
  }, [debounceSetContainerWidth])

  const gridItemWidth = useMemo(() => {
    const columns = Math.round(containerWidth / 175)
    return Math.floor((containerWidth - gap * (columns - 1)) / columns)
  }, [containerWidth])

  const onSelect = useCallback(
    (e: React.MouseEvent, data: ExtractExplorerItem<'FilePathDir' | 'FilePathWithAssetObject' | 'SearchResult'>) => {
      // 按住 cmd 键多选
      const selectIndex = items.indexOf(data)
      if (e.metaKey) {
        if (explorer.isItemSelected(data)) {
          explorer.removeSelectedItem(data)
        } else {
          explorer.addSelectedItem(data)
        }
        setLastSelectedIndex(selectIndex)
      } else if (e.shiftKey) {
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
      className="flex flex-wrap content-start items-start justify-start"
      style={{ columnGap: `${gap}px`, rowGap: '30px', padding: `${padding}px` }}
    >
      {items.map((item) => (
        <GridItem key={uniqueId(item)} data={item} onSelect={onSelect} style={{ width: `${gridItemWidth}px` }} />
      ))}
    </div>
  )

  /**
   * 列表太长需要用 react-window 来优化，只渲染可见部分，不然会很卡
   * 但是直接替换会导致 contextmenu 无法触发，需要研究一下
   * import { FixedSizeGrid as Grid } from "react-window";
   * "react-window": "^1.8"
   * "@types/react-window": "^1.8"
   */

  // const _GridItem = ({ columnIndex, rowIndex, style }: any) => {
  //   const item = items[rowIndex * 6 + columnIndex]; // Adjust according to your number of columns
  //   return (
  //     <div style={style}>
  //       <GridItem key={item.id} data={item} />
  //     </div>
  //   );
  // };
  // return (
  //   <Grid
  //     className="p-8"
  //     columnCount={6} // Adjust according to your number of columns
  //     columnWidth={150} // Adjust according to your item width
  //     height={500} // Adjust according to your grid height
  //     rowCount={Math.ceil(items.length / 3)} // Adjust according to your number of columns
  //     rowHeight={200} // Adjust according to your item height
  //     width={900} // Adjust according to your grid width
  //   >
  //     {_GridItem}
  //   </Grid>
  // )
}
