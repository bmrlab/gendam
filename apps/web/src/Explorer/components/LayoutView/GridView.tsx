'use client'
import ExplorerDraggable from '@/Explorer/components/Draggable/ExplorerDraggable'
import ExplorerDroppable, { useExplorerDroppableContext } from '@/Explorer/components/Draggable/ExplorerDroppable'
import FileThumb from '@/Explorer/components/View/FileThumb'
import RenamableItemText from '@/Explorer/components/View/RenamableItemText'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { useCallback, useMemo } from 'react'
// import styles from './GridView.module.css'

const DroppableInner: React.FC<{ data: ExplorerItem }> = ({ data }) => {
  const currentLibrary = useCurrentLibrary()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const { isDroppable } = useExplorerDroppableContext()
  const highlight = useMemo(() => {
    return explorer.isItemSelected(data) || isDroppable
  }, [data, explorer, isDroppable])

  return (
    <>
      <div className={classNames('mb-1 h-28 w-28 p-2 rounded-lg', highlight ? 'bg-slate-100' : null)}>
        <FileThumb data={data} className="w-full h-full"/>
      </div>
      {explorer.isItemSelected(data) && explorerStore.isRenaming ? (
        <RenamableItemText data={data} />
      ) : (
        <div className={classNames('w-28 rounded-lg p-1', highlight ? 'bg-blue-500 text-white' : null)}>
          <div className="line-clamp-2 max-h-[2.8em] text-center text-xs leading-[1.4em]">{data.name}</div>
        </div>
      )}
    </>
  )
}

const GridItem: React.FC<{ data: ExplorerItem }> = ({ data }) => {
  const router = useRouter()
  const currentLibrary = useCurrentLibrary()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const handleClick = (e: React.MouseEvent) => {
    // 按住 cmd 键多选
    e.stopPropagation()
    if (e.metaKey) {
      explorer.addSelectedItem(data);
    } else {
      explorer.resetSelectedItems([data])
    }
    explorerStore.reset()
  }

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
        router.push('/video-tasks')
      }
    },
    [data, explorer, router, explorerStore],
  )

  return (
    <div
      className="flex cursor-default select-none flex-col items-center justify-start"
      onClick={handleClick}
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
  return (
    <div className="flex flex-wrap content-start items-start justify-start gap-6 p-8">
      {items.map((item) => (
        <GridItem key={item.id} data={item} />
      ))}
    </div>
  )
}
