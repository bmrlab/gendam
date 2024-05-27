'use client'
import type { FilePath, FileHandlerTask } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { queryClient, rspc } from '@/lib/rspc'
import { formatBytes, formatDateTime, formatDuration } from '@/lib/utils'
import { Folder_Light } from '@gendam/assets/images'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import Image from 'next/image'
import { useCallback, useEffect, useMemo, useRef } from 'react'
import { useInspector } from './store'

const FolderDetail = ({ data }: { data: FilePath }) => {
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
    return <Icon.Close className="h-3 w-3 text-red-500" /> // 出错
  } else if (task.exitCode === 1) {
    return <Icon.Close className="h-3 w-3 text-neutral-500" /> // 取消
  } else if (task.exitCode === 0) {
    return <Icon.Check className="h-3 w-3 text-green-500" /> // 已完成
  } else if (task.startsAt) {
    // return <Icon.Cycle className="h-4 w-4 animate-spin text-orange-500" /> // 已经开始但还没结束
    return <Icon.FlashStroke className="h-3 w-3 text-orange-400" />
  } else {
    return <Icon.Clock className="h-3 w-3 text-neutral-900" />
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

const AssetObjectDetail = ({ data }: { data: FilePath }) => {
  const currentLibrary = useCurrentLibrary()

  const tasksQueryParams = useMemo(() => {
    const filter = { assetObjectId: data.assetObject?.id }
    return { filter }
  }, [data.assetObject?.id])
  const tasksQuery = rspc.useQuery(['tasks.list', tasksQueryParams], {
    enabled: !!data.assetObject?.id,
  })
  const cancelJobsMut = rspc.useMutation(['video.tasks.cancel'])
  const handleJobsCancel = useCallback(async () => {
    if (!data.assetObject?.id) {
      return
    }
    try {
      await cancelJobsMut.mutateAsync({
        assetObjectId: data.assetObject.id,
        taskTypes: null,
      })
      queryClient.invalidateQueries({
        queryKey: ['tasks.list', tasksQueryParams],
      })
    } catch (error) {}
  }, [data.assetObject?.id, cancelJobsMut, tasksQueryParams])

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
    const videoSrc = currentLibrary.getFileSrc(data.assetObject.hash, data.assetObject.mimeType!)
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
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Visual Search</div>
          {sortedTasks.some(item => item.taskType === 'frame-content-embedding' && item.exitCode === 0) ? (
            <div className="rounded-full px-2 text-xs text-green-600 bg-green-100">Ready</div>
          ) : (
            <div className="rounded-full px-2 text-xs text-orange-600 bg-orange-100">Not ready</div>
          )}
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Transcript Search</div>
          {sortedTasks.some(item => item.taskType === 'transcript-embedding' && item.exitCode === 0) ? (
            <div className="rounded-full px-2 text-xs text-green-600 bg-green-100">Ready</div>
          ) : (
            <div className="rounded-full px-2 text-xs text-orange-600 bg-orange-100">Not ready</div>
          )}
        </div>
      </div>
      <div className="bg-app-line mb-3 mt-3 h-px"></div>
      <div className="text-xs">
        <div className="text-md mt-2 flex items-center justify-between">
          <div className="font-medium">Jobs</div>
          {sortedTasks.some(task => task.exitCode === null) ? (
            <div className="group flex items-center gap-1 text-ink/60">
              <div className="px-1 transition-opacity duration-300 opacity-0 group-hover:opacity-100">Cancel pending jobs</div>
              <Button variant="ghost" size="xs" className="px-0" onClick={() => handleJobsCancel()}>
                <Icon.Close className="h-3 w-3" />
              </Button>
            </div>
          ) : null}
        </div>
        {sortedTasks.map((task) => (
          <div key={task.id} className="mt-2 flex items-center justify-between">
            <div className="text-ink/50">{(TaskItemType[task.taskType] ?? ['Unknown'])[0]}</div>
            <TaskItemStatus task={task} />
          </div>
        ))}
      </div>
      {/* blank area at the bottom */}
      <div className="mt-6"></div>
    </div>
  )
}

export default function Inspector({ data }: { data: FilePath | null }) {
  const inspector = useInspector()

  /**
   * listen to meta + I to toggle inspector
   * @todo 这个快捷键目前只是临时实现，之后应该统一的管理快捷键并且提供用户自定义的功能
   */
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.metaKey && e.key === 'i') {
        inspector.setShow(!inspector.show)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [inspector])

  return inspector.show ? (
    <div className="border-app-line h-full w-64 overflow-auto border-l">
      {/* <div onClick={() => inspector.setShow(false)}>close</div> */}
      {data ? (
        data.isDir ? (
          <FolderDetail data={data} />
        ) : data.assetObject ? (
          <AssetObjectDetail data={data} />
        ) : null
      ) : null}
    </div>
  ) : (
    <></>
  )
}
