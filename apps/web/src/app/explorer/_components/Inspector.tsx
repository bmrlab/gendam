import FileThumb from '@/Explorer/components/View/FileThumb'
import { useCurrentLibrary } from '@/lib/library'
import { Document_Light, Folder_Light } from '@muse/assets/images'
import { useExplorerContext } from '@/Explorer/hooks'
import { create } from 'zustand'
// import { useExplorerStore } from "@/Explorer/store";
import { ExplorerItem } from '@/Explorer/types'
import classNames from 'classnames'
import Image from 'next/image'
import { useMemo } from 'react'
import { formatBytes, formatDuration, formatDateTime } from '@/lib/utils'

interface InspectorState {
  show: boolean
  setShow: (show: boolean) => void
}

export const useInspector = create<InspectorState>((set) => ({
  show: false,
  setShow: (show) => set({ show }),
}))

const FolderDetail = ({ data }: { data: ExplorerItem }) => {
  return (
    <div className="p-4">
      <div className="flex items-start justify-start">
        <div className="w-12 h-12 relative">
          <Image src={Folder_Light} alt="folder" fill={true} className="object-contain" ></Image>
        </div>
        <div className="ml-3 flex-1 overflow-hidden">
          <div className="line-clamp-2 text-ink text-xs font-medium mt-1">{data.name}</div>
          {/* <div className="line-clamp-2 text-ink/50 text-xs mt-1">文件夹 {data.materializedPath}{data.name}</div> */}
        </div>
      </div>
    </div>
  )
}

const AssetObjectDetail = ({ data }: { data: ExplorerItem }) => {
  const currentLibrary = useCurrentLibrary()
  if (!data.assetObject || !data.assetObject.mediaData) {
    return
  }
  const { assetObject, assetObject: { mediaData } } = data
  return (
    <div className="p-4">
      <div className="overflow-hidden rounded-md relative">
        <video controls={true} autoPlay={true} muted loop className="w-full h-auto">
          <source src={currentLibrary.getFileSrc(assetObject.hash)} />
        </video>
      </div>
      <div className="overflow-hidden mt-3">
        <div className="line-clamp-2 break-all text-ink text-sm font-medium">{data.name}{data.name}{data.name}{data.name}{data.name}</div>
        <div className="line-clamp-2 text-ink/50 text-xs mt-1">目录 {data.materializedPath}</div>
      </div>
      <div className="h-px bg-app-line mt-6 mb-3"></div>
      <div className="text-xs">
        <div className="text-md font-medium">基本信息</div>
        <div className="flex justify-between mt-2">
          <div className="text-ink/50">大小</div>
          <div>{formatBytes(mediaData?.size ?? 0)}</div>
        </div>
        <div className="flex justify-between mt-2">
          <div className="text-ink/50">长度</div>
          <div>{formatDuration(mediaData?.duration ?? 0)}</div>
        </div>
        <div className="flex justify-between mt-2">
          <div className="text-ink/50">尺寸</div>
          <div>{`${mediaData?.width ?? 0} x ${mediaData?.height ?? 0}`}</div>
        </div>
        <div className="flex justify-between mt-2">
          <div className="text-ink/50">音频</div>
          <div>{mediaData.hasAudio ? "有" : "无"}</div>
        </div>
        <div className="flex justify-between mt-2">
          <div className="text-ink/50">创建时间</div>
          <div>{formatDateTime(data.createdAt)}</div>
        </div>
        <div className="flex justify-between mt-2">
          <div className="text-ink/50">更新时间</div>
          <div>{formatDateTime(data.updatedAt)}</div>
        </div>
      </div>
      <div className="h-px bg-app-line mt-6 mb-3"></div>
      <div className="text-xs">
        <div className="flex justify-between mt-2">
          <div className="text-ink/50">Content Hash</div>
          <div>{assetObject.hash}</div>
        </div>
      </div>
    </div>
  )
}

export default function Inspector() {
  const explorer = useExplorerContext()
  // const explorerStore = useExplorerStore()
  const inspector = useInspector()

  const item = useMemo<ExplorerItem | null>(() => {
    const selectedItems = explorer.selectedItems
    if (selectedItems.size === 1) {
      return Array.from(selectedItems)[0]
    }
    return null
  }, [explorer.selectedItems])

  return inspector.show ? (
    <div className="h-full w-64 border-l border-app-line">
      {/* <div onClick={() => inspector.setShow(false)}>close</div> */}
      {item ? (
        item.isDir ? (
          <FolderDetail data={item} />
        ) : item.assetObject ? (
          <AssetObjectDetail data={item} />
        ) : null
      ) : null}
    </div>
  ) : (
    <></>
  )
}
