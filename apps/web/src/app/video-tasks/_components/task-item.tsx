'use client'

import MuseBadge, { MuseStatus } from '@/components/Badge'
import MuseDropdownMenu, { DropdownMenuOptions } from '@/components/DropdownMenu'
import Icon from '@/components/Icon'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import { getLocalFileUrl } from '@/utils/file'
import { HTMLAttributes, useCallback, useMemo } from 'react'

export type VideoItem = {
  videoPath: string
  videoFileHash: string
  tasks: {
    taskType: string
    startsAt: string | null
    endsAt: string | null
  }[]
} & { index: number }

export type VideoTaskItemProps = {
  isSelect?: boolean
  handleClick: () => void
} & VideoItem &
  HTMLAttributes<HTMLDivElement>

export const VideoTaskStatus: React.FC<{
  task: VideoItem['tasks'][number]
}> = ({ task }) => {
  const typeToName: { [key: string]: string } = {
    // Audio: '语音转译',
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

export default function VideoTaskItem({
  videoPath,
  videoFileHash,
  tasks,
  isSelect,
  handleClick,
  ...props
}: VideoTaskItemProps) {
  const typeToName: Record<string, string> = useMemo(
    () => ({
      // Audio: '语音转译',
      // "Transcript": "语音转译",
      TranscriptEmbedding: '语音转译',
      // "FrameCaption": "图像描述",
      FrameCaptionEmbedding: '图像描述',
      // "Frame": "图像特征",
      // FrameContentEmbedding: '图像特征',
    }),
    [],
  )

  const showTask = useMemo(() => {
    return tasks.filter((task) => typeToName[task.taskType])
  }, [tasks, typeToName])

  const status = useCallback((task: VideoItem['tasks'][number]) => {
    if (task.startsAt && !task.endsAt) {
      return MuseStatus.Processing
    }
    if (task.startsAt && task.endsAt) {
      return MuseStatus.Done
    }
    return MuseStatus.Failed
  }, [])

  const moreActionOptions = useCallback((id: string) => {
    return [
      {
        label: (
          <div className="flex items-center gap-1.5">
            <Icon.cancel />
            <span>取消任务</span>
          </div>
        ),
        handleClick: () => {},
      },
      'Separator',
      {
        label: (
          <div className="flex items-center gap-1.5">
            <Icon.trash />
            <span>删除任务</span>
          </div>
        ),
        handleClick: () => {},
      },
    ] as DropdownMenuOptions[]
  }, [])

  return (
    <div
      key={videoFileHash}
      {...props}
      className={cn(
        'flex w-full justify-start border-b border-neutral-100 px-5 py-3 ',
        isSelect ? 'bg-blue-100' : 'hover:bg-neutral-100',
      )}
    >
      <div
        className="mr-4 flex h-16 w-16 cursor-pointer items-center justify-center bg-neutral-200"
        onClick={(e) => {
          handleClick()
          e.stopPropagation()
        }}
      >
        <video
          controls={false}
          autoPlay
          muted
          loop
          style={{
            width: '100%',
            height: '100%',
            objectFit: 'cover',
          }}
        >
          <source src={getLocalFileUrl(videoPath)} type="video/mp4" />
        </video>
      </div>
      <div className="mb-2 w-96 break-words">
        {/* {video.videoPath} ({video.videoFileHash}) */}
        <div className="mb-2 flex">
          <div className="mr-3">MUSE 的视频</div>
          <div className="w-32 overflow-hidden overflow-ellipsis whitespace-nowrap text-sm font-light text-neutral-400">
            {videoPath}
          </div>
        </div>
        <div className="flex text-sm font-light text-neutral-400">
          <div>00:01:04</div>
          <div className="mx-2">·</div>
          <div>10.87 MB</div>
          <div className="mx-2">·</div>
          <div>1440 x 1080</div>
        </div>
      </div>
      <div className="ml-auto flex flex-wrap items-end gap-1.5">
        {showTask.map((task, index) => (
          <div key={index} className="flex gap-1.5">
            <MuseBadge key={index} name={typeToName[task.taskType]} status={status(task)} />
            {index === showTask.length - 1 &&
              (status(task) !== MuseStatus.Processing ? (
                // TODO: real data
                <MuseDropdownMenu
                  triggerIcon={<Icon.moreVertical className="size-[25px] cursor-pointer text-[#676C77]" />}
                  options={moreActionOptions('1')}
                  contentClassName="w-[215px]"
                >
                  <Button
                    variant="ghost"
                    className="size-[25px] p-0 text-[#676C77] hover:bg-[#EBECEE] data-[state=open]:bg-[#EBECEE] data-[state=open]:text-[#262626]"
                  >
                    <span className="sr-only">Open menu</span>
                    <Icon.moreVertical className="size-[25px] cursor-pointer" />
                  </Button>
                </MuseDropdownMenu>
              ) : (
                <Icon.circleX className="size-[25px] cursor-pointer text-[#676C77]" />
              ))}
          </div>
        ))}
      </div>
    </div>
  )
}
