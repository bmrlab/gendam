'use client'
import { useExplorerDroppableContext } from '@/Explorer/components/Draggable/ExplorerDroppable'
import RenamableItemText from '@/Explorer/components/View/RenamableItemText'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { ExtractExplorerItem, uniqueId } from '@/Explorer/types'
// import { useCurrentLibrary } from '@/lib/library'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import { formatBytes, formatDateTime } from '@/lib/utils'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { HTMLAttributes, useCallback, useMemo, useState } from 'react'
import ThumbItem from '../View/ThumbItem'

const DroppableInner: React.FC<
  {
    data: ExtractExplorerItem<'FilePath' | 'SearchResult'>
    index: number
  } & HTMLAttributes<HTMLDivElement>
> = ({ data, index, className, ...props }) => {
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
  }, [filePath?.name])

  return (
    <div
      {...props}
      className={classNames(
        'text-ink flex items-center justify-start gap-2 rounded px-6 py-2',
        index % 2 === 1 && !highlight ? 'bg-app-hover' : null,
        highlight ? 'bg-accent text-white' : null,
        className,
      )}
    >
      <div className="h-8 w-8">
        <ThumbItem data={data} className="h-full w-full" variant="list" />
      </div>
      {explorer.isItemSelected(data) && explorerStore.isRenaming && data.type === 'FilePath' ? (
        <div className="max-w-96 flex-1">
          <RenamableItemText data={data} />
        </div>
      ) : (
        <div
          className={classNames(
            'flex flex-1 items-center justify-start overflow-hidden text-xs',
            highlight ? 'text-white' : null,
          )}
        >
          {/* <div className="truncate break-all">{data.name}</div> */}
          <div className="truncate whitespace-pre">{name1}</div>
          <div className="whitespace-pre">{name2}</div>
        </div>
      )}
      <div className="ml-auto" />
      {filePath && (
        <div className={classNames('w-40 text-xs text-neutral-500', highlight ? 'text-white' : null)}>
          {formatDateTime(filePath.createdAt)}
        </div>
      )}
      <div className={classNames('w-24 text-xs text-neutral-500', highlight ? 'text-white' : null)}>
        {data.assetObject ? formatBytes(data.assetObject.size) : null}
      </div>
      <div className={classNames('w-24 text-xs text-neutral-500', highlight ? 'text-white' : null)}>
        {data.type === 'FilePath' && filePath?.isDir ? 'Folder' : data.assetObject?.mimeType ?? 'unknown'}
      </div>
    </div>
  )
}

const ListItem: React.FC<
  {
    data: ExtractExplorerItem<'FilePath' | 'SearchResult'>
    index: number
    onSelect: (e: React.MouseEvent, data: ExtractExplorerItem<'FilePath' | 'SearchResult'>) => void
  } & Omit<HTMLAttributes<HTMLDivElement>, 'onSelect'>
> = ({ data, index, onSelect, ...props }) => {
  const router = useRouter()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const quickViewStore = useQuickViewStore()
  
  const filePath = useMemo(() => {
    if ('filePath' in data) return data.filePath
    return data.filePaths.at(0)
  }, [data])

  const handleDoubleClick = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      // e.stopPropagation()
      explorer.resetSelectedItems()
      explorerStore.reset()
      if (filePath?.isDir) {
        let newPath = filePath.materializedPath + filePath.name + '/'
        router.push('/explorer?dir=' + newPath)
      } else if (data.assetObject) {
        quickViewStore.open(data)
      }
    },
    [data, explorer, router, explorerStore, quickViewStore],
  )

  return (
    <ViewItem data={data} onClick={(e) => onSelect(e, data)} onDoubleClick={handleDoubleClick}>
      <DroppableInner data={data} index={index} {...props} />
    </ViewItem>
  )
}

export default function ListView({ items }: { items: ExtractExplorerItem<'FilePath' | 'SearchResult'>[] }) {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const [lastSelectIndex, setLastSelectedIndex] = useState<number>(-1)

  const onSelect = useCallback(
    (e: React.MouseEvent, data: ExtractExplorerItem<'FilePath' | 'SearchResult'>) => {
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

  return (
    <>
      <div className="border-app-line flex items-center justify-start gap-2 border-b px-10 py-2">
        <div className="text-ink pl-9 text-xs font-bold">Name</div>
        <div className="ml-auto" />
        <div className="text-ink w-40 text-xs font-bold">Created</div>
        <div className="text-ink w-24 text-xs font-bold">Size</div>
        <div className="text-ink w-24 text-xs font-bold">Type</div>
      </div>
      <div className="px-4 py-2">
        {items.map((item, index) => (
          <ListItem key={uniqueId(item)} data={item} index={index} onSelect={onSelect} className="mb-px" />
        ))}
      </div>
    </>
  )
}
