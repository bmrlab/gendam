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
import { formatBytes, formatDateTime } from '@/lib/utils'

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
        'flex items-center justify-start px-6 py-2 text-ink',
        index % 2 === 1 && !highlight ? 'bg-app-hover' : null,
        highlight ? 'bg-accent text-white' : null,
      )}
    >
      <div className="mr-3 h-8 w-8">
        <FileThumb data={data} className="w-full h-full" />
      </div>
      {explorer.isItemSelected(data) && explorerStore.isRenaming ? (
        <div className="flex-1 max-w-96 mr-2">
          <RenamableItemText data={data} />
        </div>
      ) : (
        <div className={classNames('flex-1', highlight ? 'text-white' : null)}>
          <div className="truncate text-xs break-all">{data.name}</div>
        </div>
      )}
      <div className="ml-auto" />
      <div className={classNames('text-xs text-neutral-500 w-48', highlight ? 'text-white' : null )}>
        {formatDateTime(data.createdAt)}
      </div>
      <div className={classNames('text-xs text-neutral-500 w-24', highlight ? 'text-white' : null )}>
        {data.assetObject ? formatBytes(data.assetObject.mediaData?.size ?? 0) : null}
      </div>
      <div className={classNames('text-xs text-neutral-500 w-24', highlight ? 'text-white' : null )}>
        {data.isDir ? '文件夹' : '视频'}
      </div>
    </div>
  )
}

const ListItem: React.FC<{ data: ExplorerItem; index: number }> = ({ data, index }) => {
  const router = useRouter()
  const currentLibrary = useCurrentLibrary()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const handleClick = (e: React.MouseEvent) => {
    // 按住 cmd 键多选
    e.stopPropagation()
    if (e.metaKey) {
      if (explorer.isItemSelected(data)) {
        explorer.removeSelectedItem(data)
      } else {
        explorer.addSelectedItem(data);
      }
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
        // router.push('/video-tasks')
      }
    },
    [data, explorer, router, explorerStore],
  )

  return (
    <div
      className="cursor-default select-none rounded-md overflow-hidden"
      onClick={handleClick} onDoubleClick={handleDoubleClick}
    >
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
    <>
      <div className='flex items-center justify-start px-10 py-2 border-b border-app-line'>
        <div className="text-xs text-ink font-bold pl-9">名称</div>
        <div className="ml-auto" />
        <div className="text-xs text-ink font-bold w-48">创建时间</div>
        <div className="text-xs text-ink font-bold w-24">大小</div>
        <div className="text-xs text-ink font-bold w-24">文件类型</div>
      </div>
      <div className="py-2 px-4">
        {items.map((item, index) => (
          <ListItem key={item.id} data={item} index={index} />
        ))}
      </div>
    </>
  )
}
