'use client'
import { type FilePath } from '@/lib/bindings'
import { type ExplorerItem } from '@/Explorer/types'
import { useCurrentLibrary } from '@/lib/library'
import { Document_Light, Folder_Light } from '@gendam/assets/images'
import classNames from 'classnames'
import Image from 'next/image'

type T = Extract<ExplorerItem, { filePath: FilePath }>

export default function FileThumb({ data, className }: { data: T; className?: string }) {
  const currentLibrary = useCurrentLibrary()
  return (
    <div className={classNames('overflow-hidden relative', className)}>
      {data.filePath.isDir ? (
        <Image src={Folder_Light} alt="folder" priority fill={true} className="object-contain"></Image>
      ) : data.filePath.assetObject ? (
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
          src={currentLibrary.getThumbnailSrc(data.filePath.assetObject.hash)}
          alt={data.filePath.name}
          fill={true}
          className="object-contain"
          priority
        ></Image>
      ) : (
        <Image src={Document_Light} alt="document" fill={true} priority></Image>
      )}
    </div>
  )
}
