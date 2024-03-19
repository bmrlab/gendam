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

const formatDate = (date: string) => {
  const d = new Date(date)
  return d.toLocaleDateString() + ' ' + d.toLocaleTimeString()
}

const DroppableInner: React.FC<{ data: ExplorerItem; index: number }> = ({ data, index }) => {
  const currentLibrary = useCurrentLibrary()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const { isDroppable } = useExplorerDroppableContext()
  const highlight = useMemo(() => {
    return explorer.isItemSelected(data) || isDroppable
  }, [data, explorer, isDroppable])

  return (
    <div
      className={classNames(
        'flex items-center justify-start px-6 py-1',
        index % 2 === 1 && !highlight ? 'bg-slate-100' : null,
        highlight ? 'bg-blue-600' : null,
      )}
    >
      <FileThumb data={data} className="mr-2 h-8 w-8 rounded-sm" />

      {explorer.isItemSelected(data) && explorerStore.isRenaming ? (
        <RenamableItemText data={data} />
      ) : (
        <div className={classNames('w-32', highlight ? 'text-white' : null)}>
          <div className="overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">{data.name}</div>
        </div>
      )}
      <div className="ml-auto" />
      <div className="text-xs text-neutral-500 w-48">{formatDate(data.createdAt)}</div>
      <div className="text-xs text-neutral-500 w-24">{data.isDir ? '文件夹' : '视频'}</div>
    </div>
  )
}

const ListItem: React.FC<{ data: ExplorerItem; index: number }> = ({ data, index }) => {
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
    <div className="cursor-default select-none" onClick={handleClick} onDoubleClick={handleDoubleClick}>
      <ViewItem data={data}>
        <ExplorerDroppable droppable={{ data: data }}>
          <ExplorerDraggable draggable={{ data: data }}>
            <DroppableInner data={data} index={index} />
          </ExplorerDraggable>
        </ExplorerDroppable>
      </ViewItem>
    </div>
  )
}

export default function ListView({ items }: { items: ExplorerItem[] }) {
  return (
    <div className="">
      <div className='flex items-center justify-start px-6 py-2 border-b border-neutral-200'>
        <div className="text-xs text-neutral-900 font-bold">名称</div>
        <div className="ml-auto" />
        <div className="text-xs text-neutral-900 font-bold w-48">创建时间</div>
        <div className="text-xs text-neutral-900 font-bold w-24">文件类型</div>
      </div>
      {items.map((item, index) => (
        <ListItem key={item.id} data={item} index={index} />
      ))}
    </div>
  )
}
