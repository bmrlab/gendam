import FileThumb, { ThumbnailVariant } from '@/components/FileThumb'
import { ExplorerItem } from '@/Explorer/types'
import { FilePath } from '@/lib/bindings'
import { Document_Light, Folder_Light } from '@gendam/assets/images'
import classNames from 'classnames'
import Image from 'next/image'

type T = Extract<ExplorerItem, { filePath: FilePath }>

export default function ThumbItem({
  data,
  className,
  variant,
}: {
  data: T
  className?: string
  variant: ThumbnailVariant
}) {
  return (
    <div className={classNames('relative overflow-hidden', className)}>
      {data.filePath.isDir ? (
        <Image src={Folder_Light} alt="folder" priority fill={true} className="object-contain"></Image>
      ) : data.filePath.assetObject ? (
        <FileThumb data={data.filePath.assetObject} variant={variant} className={className} />
      ) : (
        <Image src={Document_Light} alt="document" fill={true} priority></Image>
      )}
    </div>
  )
}
