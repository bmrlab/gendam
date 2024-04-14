import { useExplorerContext } from '@/Explorer/hooks'
import { useCurrentLibrary } from '@/lib/library'
import { Folder_Light } from '@muse/assets/images'
import { create } from 'zustand'
// import { useExplorerStore } from "@/Explorer/store";
import { ExplorerItem } from '@/Explorer/types'
import { formatBytes, formatDateTime, formatDuration } from '@/lib/utils'
import Image from 'next/image'
import { useRef, useEffect, useMemo } from 'react'

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
        <div className="relative h-12 w-12">
          <Image src={Folder_Light} alt="folder" fill={true} className="object-contain"></Image>
        </div>
        <div className="ml-3 flex-1 overflow-hidden">
          <div className="mt-1 line-clamp-2 text-xs font-medium text-ink">{data.name}</div>
          {/* <div className="line-clamp-2 text-ink/50 text-xs mt-1">文件夹 {data.materializedPath}{data.name}</div> */}
        </div>
      </div>
      <div className="mb-3 mt-6 h-px bg-app-line"></div>
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

const AssetObjectDetail = ({ data }: { data: ExplorerItem }) => {
  const currentLibrary = useCurrentLibrary()

  const videoRef = useRef<HTMLVideoElement|null>(null)
  useEffect(() => {
    if (!videoRef?.current || !data.assetObject?.hash) {
      return
    }
    const videoSrc = currentLibrary.getFileSrc(data.assetObject.hash)
    // 重新赋值才能在 src 变化了以后重新加载视频
    if (videoRef.current.src != videoSrc) {
      videoRef.current.src = videoSrc
    }
  }, [currentLibrary, data, videoRef])

  if (!data.assetObject || !data.assetObject.mediaData) {
    return
  }
  const {
    assetObject,
    assetObject: { mediaData },
  } = data
  return (
    <div className="p-3">
      <div className="relative h-48 w-58 overflow-hidden bg-app-overlay/50">
        <video ref={videoRef} controls autoPlay muted loop className="h-full w-full object-contain object-center">
          {/* <source src={currentLibrary.getFileSrc(assetObject.hash)} /> */}
        </video>
      </div>
      <div className="mt-3 overflow-hidden">
        <div className="line-clamp-2 break-all text-sm font-medium text-ink">{data.name}</div>
        <div className="mt-1 line-clamp-2 text-xs text-ink/50">Location {data.materializedPath}</div>
      </div>
      <div className="mb-3 mt-6 h-px bg-app-line"></div>
      <div className="text-xs">
        <div className="text-md font-medium">Information</div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Size</div>
          <div>{formatBytes(assetObject.size)}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Type</div>
          <div>{assetObject.mimeType}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Duration</div>
          <div>{formatDuration(mediaData?.duration ?? 0)}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Dimensions</div>
          <div>{`${mediaData?.width ?? 0} x ${mediaData?.height ?? 0}`}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Audio</div>
          <div>{mediaData.hasAudio ? 'Yes' : 'No'}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Created</div>
          <div>{formatDateTime(data.createdAt)}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Modified</div>
          <div>{formatDateTime(data.updatedAt)}</div>
        </div>
      </div>
      <div className="mb-3 mt-6 h-px bg-app-line"></div>
      <div className="text-xs">
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Content Hash</div>
          <div>{assetObject.hash}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Asset Object ID</div>
          <div>{assetObject.id}</div>
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
