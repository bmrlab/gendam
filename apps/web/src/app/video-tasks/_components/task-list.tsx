'use client'
import type { VideoWithTasksResult } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { cn } from '@/lib/utils'
import Image from 'next/image'
import { HTMLAttributes, useMemo } from 'react'
import { isNotDone } from './utils'
import { useBoundStore } from '../_store'
import TaskContextMenu from './TaskContextMenu'
import { WithSelectVideoItem } from './withSelect'
import EmptyList from '/public/svg/empty-list.svg'

export type VideoTasksListProps = HTMLAttributes<HTMLDivElement> & {
  data: VideoWithTasksResult[]
  isLoading: boolean
}

export default function VideoTasksList({ data, isLoading, className }: VideoTasksListProps) {
  const revealMut = rspc.useMutation('files.reveal')

  const searchKey = useBoundStore.use.searchKey()
  const taskSelected = useBoundStore.use.videoSelected()

  const filterVideos = useMemo(() => {
    return data.filter((videoFile) => {
      // TODO: 等加入更多视频信息后，需要修改搜索条件
      return videoFile?.name.includes(searchKey)
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
      {filterVideos?.map((videoFile) => {
        return (
          <TaskContextMenu
            key={videoFile.assetObject.id}
            fileHash={videoFile.assetObject.hash}
            isNotDone={isNotDone(videoFile.tasks)}
            video={videoFile}
          >
            <WithSelectVideoItem
              videoFile={videoFile}
              items={data}
              isSelect={taskSelected.some((item) => item.assetObject.hash === videoFile.assetObject.hash)}
              handleClick={() => {
                // console.log(videoFile.assetObject.id, videoFile.assetObject.hash)
                // revealMut.mutate(video.videoPath)
              }}
            />
          </TaskContextMenu>
        )
      })}
    </div>
  )
}
