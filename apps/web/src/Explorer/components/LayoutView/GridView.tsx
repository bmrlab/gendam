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
      <FileThumb data={data} className={classNames('mb-1 h-32 w-32 rounded-lg', highlight ? 'bg-slate-200' : null)} />
      {explorer.isItemSelected(data) && explorerStore.isRenaming ? (
        <RenamableItemText data={data} />
      ) : (
        <div className={classNames('w-32 rounded-lg p-1', highlight ? 'bg-blue-600 text-white' : null)}>
          <div className="line-clamp-2 h-[2.8em] text-center text-xs leading-[1.4em]">{data.name}</div>
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
    e.stopPropagation()
    explorer.resetSelectedItems([data])
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
      className="m-2 flex cursor-default select-none flex-col items-center justify-start"
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
    <div className="flex flex-wrap content-start items-start justify-start p-6">
      {items.map((item) => (
        <GridItem key={item.id} data={item} />
      ))}
    </div>
  )
}
