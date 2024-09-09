'use client'
import type { VideoWithTasksResult } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { formatBytes, formatDuration } from '@/lib/utils'
import Icon from '@gendam/ui/icons'
import classNames from 'classnames'
import Image from 'next/image'
import { HTMLAttributes, useMemo } from 'react'
import TaskDropdownMenu from './TaskDropdownMenu'
import { VideoTaskStatus } from './TaskStatus'
import { useTaskActionOptions } from './useTaskActionOptions'

export type VideoTaskItemProps = {
  videoFile: VideoWithTasksResult
  isSelect?: boolean
  handleSelect: () => void
} & HTMLAttributes<HTMLDivElement>

export default function VideoTaskItem({ videoFile, isSelect, handleSelect, ...props }: VideoTaskItemProps) {
  const currentLibrary = useCurrentLibrary()
  const { mutateAsync } = rspc.useMutation('video.tasks.cancel')

  const { options } = useTaskActionOptions([videoFile])
  const { name, assetObject, materializedPath, tasks, mediaData } = videoFile

  const moreActionOptions = useMemo(() => {
    return options.map((v) =>
      v === 'Separator'
        ? v
        : {
            label: (
              <div className="flex items-center gap-1.5">
                {v.icon}
                <span>{v.label}</span>
              </div>
            ),
            disabled: v.disabled,
            variant: v.variant,
            handleSelect: v.handleSelect,
          },
    )
  }, [options])

  return (
    <>
      <div
        {...props}
        className={classNames(
          'flex w-full cursor-default items-center justify-start gap-2 rounded-md px-4 py-3 text-sm',
          isSelect ? 'bg-accent text-white' : null,
        )}
      >
        <div
          className="bg-app-overlay relative flex size-9 cursor-default"
          onClick={(e) => {
            handleSelect()
            // e.stopPropagation()
          }}
        >
          {/* <video controls={false} autoPlay={false} muted loop className="size-full object-contain">
          <source src={currentLibrary.getFileSrc(assetObject.hash)} />
        </video> */}
          <Image
            src={currentLibrary.getThumbnailSrc(assetObject.hash, 'video')}
            alt={assetObject.hash}
            fill={true}
            className="object-cover"
            priority
          ></Image>
        </div>
        <div className="grid flex-1">
          {materializedPath ? (
            <div className="flex items-center gap-2">
              <span className="text-xs font-medium leading-4">{name}</span>
              <span className="truncate text-xs font-normal leading-4 opacity-60">{materializedPath}</span>
            </div>
          ) : (
            <div className="flex items-center gap-2">
              <span className="truncate text-xs font-normal leading-4 opacity-60">Deleted</span>
            </div>
          )}
          {mediaData?.contentType === 'video' && (
            <div className="flex w-full items-center justify-between">
              <div className="flex items-center text-xs font-normal leading-4 opacity-60">
                <span>{formatDuration(mediaData.duration ?? 0)}</span>
                <div className="mx-2">·</div>
                <span>{formatBytes(assetObject.size)}</span>
                <div className="mx-2">·</div>
                <span>{`${mediaData.width ?? 0} x ${mediaData.height ?? 0}`}</span>
                {!!mediaData.audio ? null : (
                  <>
                    <div className="mx-2">·</div>
                    <NoAudio />
                  </>
                )}
              </div>
              <div className="flex flex-wrap items-end gap-1.5">
                <VideoTaskStatus tasks={tasks}></VideoTaskStatus>
                <TaskDropdownMenu triggerIcon={<Icon.MoreVertical className="size-3" />} options={moreActionOptions}>
                  <div
                    className={classNames(
                      'inline-flex size-6 cursor-default items-center justify-center rounded border',
                      !isSelect && 'data-[state=open]:bg-app-hover',
                    )}
                  >
                    <span className="sr-only">Open menu</span>
                    <Icon.MoreVertical className="size-3" />
                  </div>
                </TaskDropdownMenu>
              </div>
            </div>
          )}
        </div>
      </div>
      <div className="bg-app-line mx-2 my-px h-px"></div>
    </>
  )
}

const NoAudio = () => {
  return (
    <div className="text-ink/50 flex items-center gap-1">
      <Icon.SpeakerSimpleX />
      <span className="text-xs font-normal leading-4">No Audio</span>
    </div>
  )
}
