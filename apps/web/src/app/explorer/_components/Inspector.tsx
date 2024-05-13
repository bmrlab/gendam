'use client'
import { useExplorerContext } from '@/Explorer/hooks'
import { ExplorerItem } from '@/Explorer/types'
import { FileHandlerTask } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { formatBytes, formatDateTime, formatDuration } from '@/lib/utils'
import { Folder_Light } from '@gendam/assets/images'
import Icon from '@gendam/ui/icons'
import Image from 'next/image'
import { useEffect, useMemo, useRef } from 'react'
import { create } from 'zustand'

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

function TaskItemStatus({ task }: { task: FileHandlerTask }) {
  if (task.exitCode !== null && task.exitCode > 1) {
    return <Icon.Close className="h-4 w-4 text-red-500" /> // 出错
  } else if (task.exitCode === 1) {
    return <Icon.Close className="h-3 w-3 text-neutral-800" /> // 取消
  } else if (task.exitCode === 0) {
    return <Icon.Check className="h-4 w-4 text-green-500" /> // 已完成
  } else if (task.startsAt) {
    return <Icon.Cycle className="h-4 w-4 animate-spin text-orange-500" /> // 已经开始但还没结束
  } else {
    return <Icon.Clock className="h-4 w-4 text-neutral-900" />
  }
}

export const TaskItemType: Record<string, [string, number]> = {
  'frame': ['Frame Processing', 1],
  'frame-content-embedding': ['Visual Indexing', 2],
  'frame-caption': ['Video Recognition', 3],
  'frame-caption-embedding': ['Description Indexing', 4],
  'audio': ['Audio Processing', 5],
  'transcript': ['Speech Recognition', 6],
  'transcript-embedding': ['Transcript Indexing', 7],
}

const AssetObjectDetail = ({ data }: { data: ExplorerItem }) => {
  const currentLibrary = useCurrentLibrary()

  const tasksQuery = rspc.useQuery(
    [
      'tasks.list',
      {
        filter: {
          assetObjectId: data.assetObject?.id,
        },
      },
    ],
    {
      enabled: !!data.assetObject?.id,
    },
  )

  const sortedTasks = useMemo(() => {
    if (!tasksQuery.data) {
      return []
    }
    return tasksQuery.data.sort((a, b) => {
      const [, indexA] = TaskItemType[a.taskType] ?? [, 0]
      const [, indexB] = TaskItemType[b.taskType] ?? [, 0]
      return indexA - indexB
    })
  }, [tasksQuery.data])

  const videoRef = useRef<HTMLVideoElement | null>(null)
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
      <div className="w-58 bg-app-overlay/50 relative h-48 overflow-hidden">
        <video ref={videoRef} controls autoPlay muted loop className="h-full w-full object-contain object-center">
          {/* <source src={currentLibrary.getFileSrc(assetObject.hash)} /> */}
        </video>
      </div>
      <div className="mt-3 overflow-hidden">
        <div className="text-ink line-clamp-2 break-all text-sm font-medium">{data.name}</div>
        <div className="text-ink/50 mt-1 line-clamp-2 text-xs">Location {data.materializedPath}</div>
      </div>
      <div className="bg-app-line mb-3 mt-3 h-px"></div>
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
      <div className="bg-app-line mb-3 mt-3 h-px"></div>
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
      <div className="bg-app-line mb-3 mt-3 h-px"></div>
      <div className="text-xs">
        <div className="text-md mt-2 font-medium">Jobs</div>
        {sortedTasks.map((task) => (
          <div key={task.id} className="mt-2 flex items-center justify-between">
            <div className="text-ink/50">{(TaskItemType[task.taskType] ?? ['Unknown',])[0]}</div>
            <TaskItemStatus task={task} />
          </div>
        ))}
      </div>
      {/* blank area at the bottom */}
      <div className="mt-6"></div>
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
    <div className="border-app-line h-full w-64 overflow-auto border-l">
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
