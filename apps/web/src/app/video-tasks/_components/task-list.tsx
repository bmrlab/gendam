'use client'
import type { VideoItem } from './task-item'
import TaskContextMenu from './task-context-menu'
import { WithSelectVideoItem } from './with-select'
import type { VideoTaskResult } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { useBoundStore } from '@/store'
import { useMemo } from 'react'

export default function VideoTasksList() {
  const { data, isLoading, error } = rspc.useQuery(['video.tasks.list'])

  const revealMut = rspc.useMutation('files.reveal')

  const taskSelected = useBoundStore.use.taskSelected()

  const videos = useMemo<VideoItem[]>(() => {
    if (isLoading) {
      return []
    }
    const groups: {
      [videoFileHash: string]: VideoItem
    } = {}
    data?.forEach((task: VideoTaskResult, index) => {
      if (!groups[task.videoFileHash]) {
        groups[task.videoFileHash] = {
          index,
          videoPath: task.videoPath,
          videoFileHash: task.videoFileHash,
          tasks: [],
        }
      }
      groups[task.videoFileHash].tasks.push({
        taskType: task.taskType,
        startsAt: task.startsAt,
        endsAt: task.endsAt,
      })
    })
    return Object.values(groups)
  }, [data, isLoading])

  if (isLoading) {
    return <div className="flex items-center justify-center px-2 py-8 text-sm text-neutral-400">正在加载...</div>
  }

  return (
    <div className="p-4">
      {videos.map((video: VideoItem) => {
        return (
          <TaskContextMenu key={video.videoFileHash} fileHash={video.videoFileHash}>
            <WithSelectVideoItem
              {...video}
              items={videos}
              isSelect={taskSelected.some((item) => item.videoFileHash === video.videoFileHash)}
              handleClick={() => {
                revealMut.mutate(video.videoPath)
              }}
            />
          </TaskContextMenu>
        )
      })}
    </div>
  )
}
