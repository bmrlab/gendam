'use client'
import type { VideoWithTasksResult } from '@/lib/bindings'
// import { rspc } from '@/lib/rspc'
import { cn } from '@/lib/utils'
import { HTMLAttributes, useMemo } from 'react'
import { isNotDone } from './utils'
import { useBoundStore } from '../_store'
import TaskContextMenu from './TaskContextMenu'
import { WithSelectVideoItem } from './withSelect'

export type VideoTasksListProps = HTMLAttributes<HTMLDivElement> & {
  data: VideoWithTasksResult[]
  // isLoading: boolean
}

export default function VideoTasksList({ data, className }: VideoTasksListProps) {
  const taskSelected = useBoundStore.use.videoSelected()
  const filterVideos = data
  // const searchKey = useBoundStore.use.searchKey()
  // const filterVideos = useMemo(() => {
  //   return data.filter((videoFile) => {
  //     // TODO: 等加入更多视频信息后，需要修改搜索条件
  //     return videoFile?.name.includes(searchKey)
  //   })
  // }, [data, searchKey])

  return (
    <div className={cn('h-full px-4', className)}>
      {filterVideos?.map((videoFile) => {
        return (
          <TaskContextMenu
            key={videoFile.assetObject.id}
          >
            <WithSelectVideoItem
              videoFile={videoFile}
              items={data}
              isSelect={taskSelected.some((item) => item.assetObject.hash === videoFile.assetObject.hash)}
              handleClick={() => {
                // console.log(videoFile.assetObject.id, videoFile.assetObject.hash)
              }}
            />
          </TaskContextMenu>
        )
      })}
    </div>
  )
}
