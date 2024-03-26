'use client'
import { ExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { Document_Light, Folder_Light } from '@muse/assets/images'
import classNames from 'classnames'
import Image from 'next/image'

export default function FileThumb({ data, className }: { data: ExplorerItem; className?: string }) {
  const currentLibrary = useCurrentLibrary()
  return (
    <div className={classNames('overflow-hidden', className)}>
      {data.isDir ? (
        <Image src={Folder_Light} alt="folder" priority></Image>
      ) : data.assetObject ? (
        <video controls={false} autoPlay muted loop style={{ width: '100%', height: '100%', objectFit: 'cover' }}>
          <source src={currentLibrary.getFileSrc(data.assetObject.hash)} />
        </video>
      ) : (
        <Image src={Document_Light} alt="document" priority></Image>
      )}
    </div>
  )
}
