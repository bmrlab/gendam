'use client'
import { useExplorerDroppableContext } from '@/Explorer/components/Draggable/ExplorerDroppable'
import FileThumb from '@/Explorer/components/View/FileThumb'
import RenamableItemText from '@/Explorer/components/View/RenamableItemText'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { uniqueId } from '@/Explorer/types'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { useCallback, useMemo } from 'react'
import { type WithFilePathExplorerItem } from './index'

const DroppableInner: React.FC<{ data: WithFilePathExplorerItem }> = ({ data }) => {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const { isDroppable } = useExplorerDroppableContext()
  const highlight = useMemo(() => {
    return explorer.isItemSelected(data) || isDroppable
  }, [data, explorer, isDroppable])

  return (
    <>
      <div className={classNames('mb-1 h-28 w-28 rounded-lg p-2', highlight ? 'bg-app-hover' : null)}>
        <FileThumb data={data} className="h-full w-full" />
      </div>
      {explorer.isItemSelected(data) && explorerStore.isRenaming ? (
        <div className="w-28">
          <RenamableItemText data={data} className="text-center" />
        </div>
      ) : (
        <div className={classNames('text-ink w-28 rounded-lg p-1', highlight ? 'bg-accent text-white' : null)}>
          <div className="line-clamp-2 max-h-[2.8em] break-all text-center text-xs leading-[1.4em]">
            {data.filePath.name}
          </div>
        </div>
      )}
    </>
  )
}

const FolderItem: React.FC<{ data: WithFilePathExplorerItem }> = ({ data }) => {
  const router = useRouter()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  // data.isDir is always true
  const handleDoubleClick = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      explorer.resetSelectedItems()
      explorerStore.reset()
      let newPath = data.filePath.materializedPath + data.filePath.name + '/'
      router.push('/explorer?dir=' + newPath)
    },
    [data, explorer, router, explorerStore],
  )

  const onSelect = useCallback(
    (e: React.MouseEvent, data: WithFilePathExplorerItem) => {
      explorer.resetSelectedItems([data])
      explorerStore.reset()
    },
    [explorer, explorerStore],
  )

  return (
    <ViewItem data={data} onClick={(e) => onSelect(e, data)} onDoubleClick={handleDoubleClick}>
      <DroppableInner data={data} />
    </ViewItem>
  )
}

export default function Folders({ items }: { items: WithFilePathExplorerItem[] }) {
  return (
    // <div className="w-full overflow-hidden">
    <div className="flex flex-wrap content-start items-start justify-start gap-6 overflow-scroll p-8">
      {items.map((item) => (
        <FolderItem key={uniqueId(item)} data={item} />
      ))}
    </div>
    // </div>
  )
}
