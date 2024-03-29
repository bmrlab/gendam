'use client'
import { MuseStatus, MuseTaskBadge } from '@/components/Badge'
import MuseDropdownMenu, { DropdownMenuOptions } from '@/components/DropdownMenu'
import Icon from '@/components/Icon'
import type { VideoWithTasksResult } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { cn, formatBytes, formatDuration } from '@/lib/utils'
import { Button } from '@muse/ui/v1/button'
import Image from 'next/image'
import { HTMLAttributes, useCallback, useMemo } from 'react'
import { VIDEO_DIMENSION, getTaskStatus } from './utils'
// import classNames from 'classnames'

export type VideoTaskItemProps = {
  videoFile: VideoWithTasksResult
  isSelect?: boolean
  handleClick: () => void
} & HTMLAttributes<HTMLDivElement>

export default function VideoTaskItem({
  videoFile: { name, assetObject, materializedPath, tasks, mediaData },
  isSelect,
  handleClick,
  ...props
}: VideoTaskItemProps) {
  const currentLibrary = useCurrentLibrary()

  const { mutateAsync } = rspc.useMutation('video.tasks.cancel')

  const showTask = useMemo(() => {
    const tasksWithIndex = tasks
      .map((task) => {
        const [taskName, index, showOnComplete] = VIDEO_DIMENSION[task.taskType] ?? []
        const status = getTaskStatus(task)
        return { task, taskName, index, showOnComplete, status }
      })
      .filter(({ task, taskName, index, showOnComplete, status }) => {
        if (status === MuseStatus.Processing || status === MuseStatus.Failed) {
          return true
        }
        if (status === MuseStatus.Done && showOnComplete) {
          return true
        }
        return false
      })
    return tasksWithIndex.sort((a, b) => a.index - b.index)
  }, [tasks])

  const isProcessing = useMemo(() => {
    return !!tasks.find((task) => getTaskStatus(task) === MuseStatus.Processing)
  }, [tasks])

  const hasAudio = useMemo(() => {
    return tasks.some((task) => task.taskType === 'Audio' && task.exitCode === 0)
  }, [tasks])

  const moreActionOptions = useCallback(
    (id: string, isProcessing = false) => {
      const processItem = isProcessing
        ? [
            {
              label: (
                <div className="flex items-center gap-1.5">
                  <Icon.cancel />
                  <span>取消任务</span>
                </div>
              ),
              handleClick: () => {
                console.log('cancel task', assetObject.id)
                mutateAsync({ assetObjectId: assetObject.id })
              },
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
    },
    [assetObject.id, mutateAsync],
  )

  return (
    <div
      {...props}
      className={cn(
        'flex w-full items-center justify-start gap-2 border-b border-[#EBECEE] px-4 py-3 ',
        isSelect ? 'bg-blue-100' : 'hover:bg-neutral-100',
      )}
    >
      <div
        className="relative flex size-9 cursor-pointer bg-[#F6F7F9]"
        onClick={(e) => {
          handleClick()
          e.stopPropagation()
        }}
      >
        {/* <video controls={false} autoPlay={false} muted loop className="size-full object-contain">
          <source src={currentLibrary.getFileSrc(assetObject.hash)} />
        </video> */}
        <Image
          src={currentLibrary.getThumbnailSrc(assetObject.hash)}
          alt={assetObject.hash}
          fill={true}
          className="object-cover"
          priority
        ></Image>
      </div>
      <div className="grid flex-1">
        {materializedPath ? (
          <div className="flex items-center gap-2">
            <span className="text-[13px] font-medium leading-[18px] text-[#323438]">{name}</span>
            <span className="truncate text-[12px] font-normal leading-4 text-[#95989F]">{materializedPath}</span>
          </div>
        ) : (
          <div className="flex items-center gap-2">
            <span className="truncate text-[12px] font-normal leading-4 text-[#95989F]">已删除</span>
          </div>
        )}
        <div className="flex w-full items-center justify-between">
          <div className="flex items-center text-[12px] font-normal leading-4 text-[#95989F]">
            <span>{formatDuration(mediaData?.duration ?? 0)}</span>
            <div className="mx-2">·</div>
            <span>{formatBytes(mediaData?.size ?? 0)}</span>
            <div className="mx-2">·</div>
            <span>{`${mediaData?.width ?? 0} x ${mediaData?.height ?? 0}`}</span>
            {hasAudio ? null : (
              <>
                <div className="mx-2">·</div>
                <NoAudio />
              </>
            )}
            {/*<div className="mx-2">·</div>*/}
            {/*<span>已取消</span>*/}
          </div>
          <div className="flex flex-wrap items-end gap-1.5">
            {showTask.map(({ taskName, index, status }) => (
              <MuseTaskBadge key={index} name={taskName} status={status} />
            ))}
            <MuseDropdownMenu
              triggerIcon={<Icon.moreVertical className="size-[25px] cursor-pointer text-[#676C77]" />}
              options={moreActionOptions('1', isProcessing)}
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
            {/* <Icon.circleX className="size-[25px] cursor-pointer text-[#676C77]" /> */}
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
