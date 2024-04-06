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
import { useRouter } from 'next/navigation'
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

const FolderItem: React.FC<{ data: ExplorerItem }> = ({ data }) => {
  const router = useRouter()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  // data.isDir is always true
  const handleDoubleClick = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      explorer.resetSelectedItems()
      explorerStore.reset()
      let newPath = data.materializedPath + data.name + '/'
      router.push('/explorer?dir=' + newPath)
    },
    [data, explorer, router, explorerStore],
  )

  return (
    <div
      className="flex cursor-default select-none flex-col items-center justify-start"
      onClick={(e) => e.stopPropagation()}
      onDoubleClick={handleDoubleClick}
    >
      <ViewItem data={data}>
        <ExplorerDroppable droppable={{ data: data }}>
          <DroppableInner data={data} />
        </ExplorerDroppable>
      </ViewItem>
    </div>
  )
}

const MediaItem: React.FC<{
  data: ExplorerItem,
  onSelect: (e: React.MouseEvent, data: ExplorerItem) => void
}> = ({ data, onSelect }) => {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const quickViewStore = useQuickViewStore()

  const handleDoubleClick = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      explorer.resetSelectedItems()
      explorerStore.reset()
      quickViewStore.open(data)
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

export default function MediaView({ items }: { items: ExplorerItem[] }) {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const [lastSelectIndex, setLastSelectedIndex] = useState<number>(-1)

  const [folders, medias] = useMemo(() => {
    return items.reduce<[ExplorerItem[], ExplorerItem[]]>(([folders, medias], item) => {
      if (item.isDir) {
        folders.push(item)
      } else {
        medias.push(item)
      }
      return [folders, medias]
    }, [[], []])
  }, [items])

  const onSelect = useCallback((e: React.MouseEvent, data: ExplorerItem) => {
    // 只处理 medias 的选择
    const selectIndex = medias.indexOf(data)
    if (e.metaKey) {
      if (explorer.isItemSelected(data)) {
        explorer.removeSelectedItem(data)
      } else {
        explorer.addSelectedItem(data);
      }
      setLastSelectedIndex(selectIndex)
    } else if (e.shiftKey) {
      // TODO: 这里要改一下，explorer.selectedItems 也可能包含了文件夹的，比如通过右键选择的
      if (explorer.selectedItems.size > 0 && lastSelectIndex >= 0) {
        const start = Math.min(lastSelectIndex, selectIndex)
        const end = Math.max(lastSelectIndex, selectIndex)
        explorer.resetSelectedItems(medias.slice(start, end + 1))
      }
    } else {
      explorer.resetSelectedItems([data])
      setLastSelectedIndex(selectIndex)
    }
    explorerStore.reset()
  }, [explorer, explorerStore, medias, lastSelectIndex])

  return (
    <>
      <div className="flex flex-wrap content-start items-start justify-start gap-6 p-8">
        {folders.map((item) => (
          <FolderItem key={item.id} data={item} />
        ))}
      </div>
      <div className="h-px my-2 bg-app-line"></div>
      <div className="flex flex-wrap content-start items-start justify-start gap-6 p-8">
        {medias.map((item) => (
          <MediaItem key={item.id} data={item} onSelect={onSelect} />
        ))}
      </div>
    </>
  )
}
