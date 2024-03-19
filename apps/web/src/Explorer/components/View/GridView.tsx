'use client'
import classNames from 'classnames'
import ViewItem from '@/Explorer/components/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { Document_Light, Folder_Light } from '@muse/assets/images'
import Image from 'next/image'
import { useCallback, useEffect, useRef } from 'react'
import ExplorerDraggable from '../ExplorerDraggable'
import ExplorerDroppable from '../ExplorerDroppable'
import styles from './GridView.module.css'

const RenamableItemText = ({ data }: { data: ExplorerItem }) => {
  const explorerStore = useExplorerStore()
  const explorer = useExplorerContext()
  const renameMut = rspc.useMutation(['assets.rename_file_path'])
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.focus()
      inputRef.current.value = data.name
      inputRef.current.select()
    }
  }, [inputRef, data])

  const handleInputSubmit = useCallback(
    (e: React.FormEvent) => {
      if (!inputRef.current?.value) {
        return
      }
      if (!explorer.parentPath) {
        // TODO: explorer.parentPath 到这一步不应该是空的，然后 data.id 如果存在，其实可以忽略 parentPath 参数
        return
      }
      console.log('input complete')
      e.preventDefault()
      // explorerStore.setIsRenaming(false)
      explorerStore.reset()
      renameMut.mutate({
        id: data.id,
        path: explorer.parentPath,
        isDir: data.isDir,
        oldName: data.name,
        newName: inputRef.current.value,
      })
    },
    [explorer.parentPath, explorerStore, renameMut, data.id, data.isDir, data.name],
  )

  return (
    <form className="w-32 pt-1" onSubmit={handleInputSubmit}>
      <input
        ref={inputRef}
        className="block w-full rounded-sm border-2 border-blue-600 px-2 py-1 text-center text-xs"
        type="text"
        onClick={(e) => e.stopPropagation()}
        onDoubleClick={(e) => e.stopPropagation()}
        onBlur={() => {
          console.log('on blur, but do nothing, press enter to submit')
        }}
      />
    </form>
  )
}

const GridItem: React.FC<{ data: ExplorerItem }> = ({ data }) => {
  const currentLibrary = useCurrentLibrary()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const handleClick = (e: React.MouseEvent) => {
    e.stopPropagation()
    explorer.resetSelectedItems([data])
    explorerStore.reset()
  }

  return (
    <div
      className={classNames(
        'm-2 flex cursor-default select-none flex-col items-center justify-start',
        explorer.isItemSelected(data) && styles['selected']
      )}
      onClick={handleClick}
    >
      <ViewItem data={data}>
        <ExplorerDroppable droppable={{ data: data }}>
          <ExplorerDraggable draggable={{ data: data }}>
            <div className={`${styles['image']} h-32 w-32 overflow-hidden rounded-lg`}>
              {data.isDir ? (
                <Image src={Folder_Light} alt="folder" priority></Image>
              ) : data.assetObject ? (
                <video
                  controls={false}
                  autoPlay
                  muted
                  loop
                  style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                >
                  <source src={currentLibrary.getFileSrc(data.assetObject.hash)} type="video/mp4" />
                </video>
              ) : (
                <Image src={Document_Light} alt="document" priority></Image>
              )}
            </div>
            {explorer.isItemSelected(data) && explorerStore.isRenaming ? (
              <RenamableItemText data={data} />
            ) : (
              <div className={`${styles['title']} mt-1 w-32 rounded-lg p-1`}>
                <div className="line-clamp-2 h-[2.8em] text-center text-xs leading-[1.4em]">{data.name}</div>
              </div>
            )}
          </ExplorerDraggable>
        </ExplorerDroppable>
      </ViewItem>
    </div>
  )
}

export default function GridView({ items }: { items: ExplorerItem[] }) {
  return (
    <div className="flex flex-wrap content-start items-start justify-start p-6">
      {items.map((item) => <GridItem key={item.id} data={item} />)}
    </div>
  )
}
