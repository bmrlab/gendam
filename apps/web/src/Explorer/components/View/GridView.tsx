'use client'
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

export default function GridView({ items }: { items: ExplorerItem[] }) {
  const currentLibrary = useCurrentLibrary()
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  return (
    <div className="flex flex-wrap content-start items-start justify-start p-6">
      {items.map((item) => (
        <div
          key={item.id}
          className={`m-2 flex cursor-default select-none flex-col items-center justify-start
            ${explorer.isItemSelected(item) && styles['selected']}`}
          onClick={(e) => {
            e.stopPropagation() // FIXME: 会导致点了文件夹以后右键菜单无法被关闭
            explorer.resetSelectedItems([item])
            explorerStore.reset()
          }}
        >
          <ExplorerDraggable draggable={{ data: item }}>
            <ExplorerDroppable droppable={{ data: item }}>
              <ViewItem data={item}>
                <div className={`${styles['image']} h-32 w-32 overflow-hidden rounded-lg`}>
                  {item.isDir ? (
                    <Image src={Folder_Light} alt="folder" priority></Image>
                  ) : item.assetObject ? (
                    <video
                      controls={false}
                      autoPlay
                      muted
                      loop
                      style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                    >
                      <source src={currentLibrary.getFileSrc(item.assetObject.hash)} type="video/mp4" />
                    </video>
                  ) : (
                    <Image src={Document_Light} alt="folder" priority></Image>
                  )}
                </div>
                {explorer.isItemSelected(item) && explorerStore.isRenaming ? (
                  <RenamableItemText data={item} />
                ) : (
                  <div className={`${styles['title']} mb-2 mt-1 w-32 rounded-lg p-1`}>
                    <div className="line-clamp-2 h-[2.8em] text-center text-xs leading-[1.4em]">{item.name}</div>
                  </div>
                )}
              </ViewItem>
            </ExplorerDroppable>
          </ExplorerDraggable>
        </div>
      ))}
    </div>
  )
}
