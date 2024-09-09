import FileThumb from '@/components/FileContent/Thumb'
import { ThumbnailVariant } from '@/components/FileContent/Thumb/types'
import { ExtractExplorerItem } from '@/Explorer/types'
import { Document_Light, Folder_Light } from '@gendam/assets/images'
import classNames from 'classnames'
import Image from 'next/image'

export default function ThumbItem({
  data,
  className,
  variant,
}: {
  data: ExtractExplorerItem<'FilePath' | 'SearchResult'>
  className?: string
  variant: ThumbnailVariant
}) {
  return (
    <div className={classNames('relative overflow-hidden', className)}>
      {data.type === 'FilePath' && data.filePath.isDir ? (
        <Image src={Folder_Light} alt="folder" priority fill={true} className="object-contain"></Image>
      ) : data.assetObject ? (
        <FileThumb data={data} variant={variant} className={className} />
      ) : (
        <Image src={Document_Light} alt="document" fill={true} priority></Image>
      )}
    </div>
  )
}
