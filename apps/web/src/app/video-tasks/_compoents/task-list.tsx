'use client'

import TaskContextMenu from '@/app/video-tasks/_compoents/task-context-menu'
import { WithSelectVideoItem } from '@/hoc/with-select'
import type { VideoTaskResult } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { useBoundStore } from '@/store'
import { FC, useMemo } from 'react'

export type VideoItem = {
  videoPath: string
  videoFileHash: string
  tasks: {
    taskType: string
    startsAt: string | null
    endsAt: string | null
  }[]
} & { index: number }

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

export const VideoTaskStatus: FC<{
  task: VideoItem['tasks'][number]
}> = ({ task }) => {
  const typeToName: { [key: string]: string } = {
    Audio: '语音转译',
    // "Transcript": "语音转译",
    TranscriptEmbedding: '语音转译',
    // "FrameCaption": "图像描述",
    FrameCaptionEmbedding: '图像描述',
    // "Frame": "图像特征",
    FrameContentEmbedding: '图像特征',
  }
  if (!typeToName[task.taskType]) {
    return <></>
  }
  if (!task.startsAt) {
    return (
      <div className="mr-2 overflow-hidden overflow-ellipsis whitespace-nowrap rounded-full bg-neutral-100/80 px-3 py-1 text-xs font-light text-neutral-600">
        {typeToName[task.taskType]}
      </div>
    )
  } else if (task.startsAt && !task.endsAt) {
    return (
      <div className="mr-2 overflow-hidden overflow-ellipsis whitespace-nowrap rounded-full bg-orange-100/80 px-3 py-1 text-xs font-light text-orange-600">
        {typeToName[task.taskType]}
      </div>
    )
  } else if (task.startsAt && task.endsAt) {
    return (
      <div className="mr-2 overflow-hidden overflow-ellipsis whitespace-nowrap rounded-full bg-green-100/80 px-3 py-1 text-xs font-light text-green-600">
        {typeToName[task.taskType]}
      </div>
    )
  } else {
    return <></>
  }
}
