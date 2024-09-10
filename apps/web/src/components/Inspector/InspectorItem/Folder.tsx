import { RawFilePath } from '@/Explorer/types'
import { formatDateTime } from '@/lib/utils'
import { Folder_Light } from '@gendam/assets/images'
import Image from 'next/image'

export default function FolderDetail({ data }: { data: RawFilePath }) {
  return (
    <div className="p-4">
      <div className="flex items-start justify-start">
        <div className="relative h-12 w-12">
          <Image src={Folder_Light} alt="folder" fill={true} className="object-contain"></Image>
        </div>
        <div className="ml-3 flex-1 overflow-hidden">
          <div className="text-ink mt-1 line-clamp-2 text-xs font-medium">{data.name}</div>
          {/* <div className="line-clamp-2 text-ink/50 text-xs mt-1">文件夹 {data.materializedPath}{data.name}</div> */}
        </div>
      </div>
      <div className="bg-app-line mb-3 mt-6 h-px"></div>
      <div className="text-xs">
        <div className="text-md font-medium">Information</div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Created</div>
          <div>{formatDateTime(data.createdAt)}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Modified</div>
          <div>{formatDateTime(data.updatedAt)}</div>
        </div>
      </div>
    </div>
  )
}
