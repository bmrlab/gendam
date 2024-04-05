'use client'
import ExplorerDraggable from '@/Explorer/components/Draggable/ExplorerDraggable'
import ExplorerDroppable, { useExplorerDroppableContext } from '@/Explorer/components/Draggable/ExplorerDroppable'
import FileThumb from '@/Explorer/components/View/FileThumb'
import RenamableItemText from '@/Explorer/components/View/RenamableItemText'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
// import { useCurrentLibrary } from '@/lib/library'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { useCallback, useMemo, useState } from 'react'
// import styles from './GridView.module.css'

const DroppableInner: React.FC<{ data: ExplorerItem }> = ({ data }) => {
  // const currentLibrary = useCurrentLibrary()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const { isDroppable } = useExplorerDroppableContext()
  const highlight = useMemo(() => {
    return explorer.isItemSelected(data) || isDroppable
  }, [data, explorer, isDroppable])

  return (
    <>
      <div className={classNames('mb-1 h-28 w-28 p-2 rounded-lg', highlight ? 'bg-app-hover' : null)}>
        <FileThumb data={data} className="w-full h-full"/>
      </div>
      {explorer.isItemSelected(data) && explorerStore.isRenaming ? (
        <div className="w-28">
          <RenamableItemText data={data} className="text-center" />
        </div>
      ) : (
        <div className={classNames(
          'w-28 rounded-lg p-1 text-ink',
          highlight ? 'bg-accent text-white' : null
        )}>
          <div className="line-clamp-2 max-h-[2.8em] text-center text-xs leading-[1.4em] break-all">{data.name}</div>
        </div>
      )}
    </>
  )
}

const GridItem: React.FC<{
  data: ExplorerItem,
  onSelect: (e: React.MouseEvent, data: ExplorerItem) => void
}> = ({ data, onSelect }) => {
  const router = useRouter()
  // const currentLibrary = useCurrentLibrary()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  // const processVideoMut = rspc.useMutation(['assets.process_video_asset'])
  const handleDoubleClick = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      // e.stopPropagation()
      explorer.resetSelectedItems()
      explorerStore.reset()
      if (data.isDir) {
        let newPath = explorer.parentPath + data.name + '/'
        router.push('/explorer?dir=' + newPath)
      } else {
        // processVideoMut.mutate(data.id)
        // router.push('/video-tasks')
      }
    },
    [data, explorer, router, explorerStore],
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

export default function GridView({ items }: { items: ExplorerItem[] }) {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const [lastSelectIndex, setLastSelectedIndex] = useState<number>(-1)

  const onSelect = useCallback((e: React.MouseEvent, data: ExplorerItem) => {
    // 按住 cmd 键多选
    const selectIndex = items.indexOf(data)
    if (e.metaKey) {
      if (explorer.isItemSelected(data)) {
        explorer.removeSelectedItem(data)
      } else {
        explorer.addSelectedItem(data);
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
  }, [explorer, explorerStore, items, lastSelectIndex])

  return (
    <div className="flex flex-wrap content-start items-start justify-start gap-6 p-8">
      {items.map((item) => (
        <GridItem key={item.id} data={item} onSelect={onSelect} />
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
