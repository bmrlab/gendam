'use client'
import { hasProcessing } from '@/app/video-tasks/_components/utils'
import { rspc } from '@/lib/rspc'
import { cn } from '@/lib/utils'
import { useBoundStore } from '@/store'
import Image from 'next/image'
import { HTMLAttributes, useMemo } from 'react'
import TaskContextMenu from './task-context-menu'
import type { VideoItem } from './task-item'
import { WithSelectVideoItem } from './with-select'
import EmptyList from '/public/svg/empty-list.svg'

export type VideoTasksListProps = HTMLAttributes<HTMLDivElement> & {
  data?: VideoItem[]
  isLoading: boolean
}

export default function VideoTasksList({ data, isLoading, className }: VideoTasksListProps) {
  const revealMut = rspc.useMutation('files.reveal')

  const searchKey = useBoundStore.use.searchKey()
  const taskSelected = useBoundStore.use.taskSelected()

  const filterVideos = useMemo(() => {
    return data?.filter((video) => {
      // TODO: 等加入更多视频信息后，需要修改搜索条件
      return video.videoPath.includes(searchKey)
    })
  }, [data, searchKey])

  if (isLoading) {
    return (
      <div className="relative h-full">
        <div className="absolute left-1/2 top-1/2 grid translate-x-[-50%] translate-y-[-50%]">
          <Image src={EmptyList} width={250} height={250} alt="empty-list" />
          <p className="mt-6 text-center text-[20px] font-medium leading-6 text-[#262626]">拖放或粘贴视频到此区域</p>
          <p className="mt-2 text-center text-[14px] leading-5 text-[#AAADB2]">多个视频/视频文件夹</p>
        </div>
      </div>
    )
  }

  return (
    <div className={cn('h-full px-4', className)}>
      {filterVideos?.map((video: VideoItem) => {
        return (
          <TaskContextMenu
            key={video.videoFileHash}
            fileHash={video.videoFileHash}
            isProcessing={hasProcessing(video.tasks)}
          >
            <WithSelectVideoItem
              {...video}
              items={data ?? []}
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
