'use client'
import AssetContextMenu from '@/components/AssetContextMenu'
import { CurrentLibrary } from '@/lib/library'
import { Document_Light, Folder_Light } from '@muse/assets/icons'
import Image from 'next/image'
import { useContext } from 'react'
import { useExplorerContext } from '../Context'
import { ExplorerItem } from '../types'
import styles from './GridView.module.css'

export default function GridView({ items }: { items: ExplorerItem[] }) {
  const currentLibrary = useContext(CurrentLibrary)
  const explorer = useExplorerContext()

  return (
    <div className="flex flex-wrap content-start items-start justify-start p-6">
      {items.map((item) => (
        <div
          key={item.id}
          className={`m-2 flex cursor-default select-none flex-col items-center justify-start
            ${explorer.isItemSelected(item) && styles['selected']}`}
          onClick={(e) => {
            e.stopPropagation()  // FIXME: 会导致点了文件夹以后右键菜单无法被关闭
            explorer.resetSelectedItems([item])
          }}
        >
          <AssetContextMenu item={item}>
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
            <div className={`${styles['title']} mb-2 mt-1 w-32 rounded-lg p-1`}>
              <div className="line-clamp-2 h-[2.8em] text-center text-xs leading-[1.4em]">{item.name}</div>
            </div>
          </AssetContextMenu>
        </div>
      ))}
    </div>
  )
}
