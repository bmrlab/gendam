'use client'
import { ExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { Document_Light, Folder_Light } from '@muse/assets/images'
import classNames from 'classnames'
import Image from 'next/image'

export default function FileThumb({ data, className }: { data: ExplorerItem; className?: string }) {
  const currentLibrary = useCurrentLibrary()
  return (
    <div className={classNames('overflow-hidden relative', className)}>
      {data.isDir ? (
        <Image src={Folder_Light} alt="folder" priority></Image>
      ) : data.assetObject ? (
        // <video
        //   controls={false}
        //   autoPlay={false}
        //   muted
        //   loop
        //   style={{ width: '100%', height: '100%', objectFit: 'cover' }}
        // >
        //   <source src={currentLibrary.getFileSrc(data.assetObject.hash)} />
        // </video>
        <Image
          src={currentLibrary.getThumbnailSrc(data.assetObject.hash)}
          alt={data.name}
          fill={true}
          className="object-cover"
          priority
        ></Image>
      ) : (
        <Image src={Document_Light} alt="document" fill={true} priority></Image>
      )}
    </div>
  )
}
