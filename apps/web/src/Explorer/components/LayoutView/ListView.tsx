'use client'
import ExplorerDraggable from '@/Explorer/components/Draggable/ExplorerDraggable'
import ExplorerDroppable, { useExplorerDroppableContext } from '@/Explorer/components/Draggable/ExplorerDroppable'
import RenamableItemText from '@/Explorer/components/View/RenamableItemText'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { Document_Light, Folder_Light } from '@muse/assets/images'
import classNames from 'classnames'
import Image from 'next/image'
import { useRouter } from 'next/navigation'
import { useCallback, useMemo } from 'react'

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
      <div className={classNames('mr-2 h-8 w-8 overflow-hidden rounded-lg')}>
        {data.isDir ? (
          <Image src={Folder_Light} alt="folder" priority></Image>
        ) : data.assetObject ? (
          <video controls={false} autoPlay muted loop style={{ width: '100%', height: '100%', objectFit: 'cover' }}>
            <source src={currentLibrary.getFileSrc(data.assetObject.hash)} type="video/mp4" />
          </video>
        ) : (
          <Image src={Document_Light} alt="document" priority></Image>
        )}
      </div>
      {explorer.isItemSelected(data) && explorerStore.isRenaming ? (
        <RenamableItemText data={data} />
      ) : (
        <div className={classNames('w-32', highlight ? 'text-white' : null)}>
          <div className="overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">{data.name}</div>
        </div>
      )}
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
      {items.map((item, index) => (
        <ListItem key={item.id} data={item} index={index} />
      ))}
    </div>
  )
}
