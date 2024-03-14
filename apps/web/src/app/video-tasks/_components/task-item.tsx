'use client'
import { HTMLAttributes, useCallback, useMemo, useContext } from 'react'
import { VIDEO_DIMENSION } from '@/app/video-tasks/_components/utils'
import { MuseStatus, MuseTaskBadge } from '@/components/Badge'
import MuseDropdownMenu, { DropdownMenuOptions } from '@/components/DropdownMenu'
import Icon from '@/components/Icon'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'
import { CurrentLibrary } from "@/lib/library";
import type { VideoWithTasksResult } from '@/lib/bindings'

export type VideoTaskItemProps = {
  videoFile: VideoWithTasksResult,
  isSelect?: boolean
  handleClick: () => void
} & HTMLAttributes<HTMLDivElement>

export default function VideoTaskItem({
  videoFile: {
    name,
    assetObjectId,
    materializedPath,
    assetObjectHash,
    tasks,
  },
  isSelect,
  handleClick,
  ...props
}: VideoTaskItemProps) {
  const currentLibrary = useContext(CurrentLibrary);

  const showTask = useMemo(() => {
    return tasks.filter((task) => VIDEO_DIMENSION[task.taskType])
  }, [tasks])

  const status = useCallback((task: VideoWithTasksResult['tasks'][number]) => {
    if (task.startsAt && !task.endsAt) {
      return MuseStatus.Processing
    }
    if (task.startsAt && task.endsAt) {
      return MuseStatus.Done
    }
    return MuseStatus.None
    // return MuseStatus.Failed
  }, [])

  const moreActionOptions = useCallback((id: string, isProcessing = false) => {
    const processItem = isProcessing
      ? [
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
        ]
      : []
    return [
      ...processItem,
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
      {...props}
      className={cn(
        'flex w-full items-center justify-start gap-2 border-b border-[#EBECEE] px-4 py-3 ',
        isSelect ? 'bg-blue-100' : 'hover:bg-neutral-100',
      )}
    >
      <div
        className="flex size-9 cursor-pointer bg-[#F6F7F9]"
        onClick={(e) => {
          handleClick()
          e.stopPropagation()
        }}
      >
        <video controls={false} autoPlay muted loop className="size-full object-contain">
          <source src={currentLibrary.getFileSrc(assetObjectId)} type="video/mp4" />
        </video>
      </div>
      <div className="grid flex-1">
        <div className="flex items-center gap-2">
          <span className="text-[13px] font-medium leading-[18px] text-[#323438]">{name}</span>
          <span className="truncate text-[12px] font-normal leading-4 text-[#95989F]">{materializedPath}</span>
        </div>
        <div className="flex w-full items-center justify-between">
          <div className="flex items-center text-[12px] font-normal leading-4 text-[#95989F]">
            <span>00:01:04</span>
            <div className="mx-2">·</div>
            <span>10.87 MB</span>
            <div className="mx-2">·</div>
            <span>1440 x 1080</span>
            <div className="mx-2">·</div>
            <NoAudio />
            {/* <div className="mx-2">·</div>
            <span>已取消</span> */}
          </div>
          <div className="flex flex-wrap items-end gap-1.5">
            {showTask.map((task, index) => (
              <div key={index} className="flex gap-1.5">
                <MuseTaskBadge key={index} name={VIDEO_DIMENSION[task.taskType]} status={status(task)} />
                {index === showTask.length - 1 &&
                  (status(task) !== MuseStatus.Processing ? (
                    // TODO: real data
                    <MuseDropdownMenu
                      triggerIcon={<Icon.moreVertical className="size-[25px] cursor-pointer text-[#676C77]" />}
                      options={moreActionOptions('1', status(task) === MuseStatus.Processing)}
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
      </div>
    </div>
  )
}

const NoAudio = () => {
  return (
    <div className="flex items-center gap-[3px] text-[#95989F]">
      <Icon.audio />
      <span className="text-[12px] font-normal leading-4">无音轨</span>
    </div>
  )
}
