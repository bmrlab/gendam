'use client'
import AssetContextMenu from '@/components/AssetContextMenu'
import { CurrentLibrary } from '@/lib/library'
import { Document_Light, Folder_Light } from '@muse/assets/icons'
import Image from 'next/image'
import { useCallback, useContext, useState } from 'react'
import { ExplorerItem } from '../types'
import styles from './GridView.module.css'

export default function GridView({ items }: { items: ExplorerItem[] }) {
  const currentLibrary = useContext(CurrentLibrary)

  let [selectedId, setSelectedId] = useState<number | null>(null)

  return (
    <div
      className="flex flex-1 flex-wrap content-start items-start justify-start p-6"
      onClick={() => setSelectedId(null)}
    >
      {items.map((item) => (
        <div
          key={item.id}
          className={`m-2 flex cursor-default select-none flex-col items-center justify-start
            ${selectedId === item.id && styles['selected']}`}
          onClick={(e) => {
            e.stopPropagation()
            setSelectedId(item.id)
          }}
          onDoubleClick={(e) => {
            // e.stopPropagation()
            setSelectedId(null)
          }}
        >
          <AssetContextMenu item={item}>
            <div className={`${styles['image']} h-32 w-32 overflow-hidden rounded-lg`}>
              {item.isDir ? (
                <Image src={Folder_Light} alt="folder" priority></Image>
              ) : item.assetObject ? (
                <video controls={false} autoPlay muted loop style={{ width: '100%', height: '100%', objectFit: 'cover' }}>
                  <source src={currentLibrary.getFileSrc(item.assetObject.id)} type="video/mp4" />
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
