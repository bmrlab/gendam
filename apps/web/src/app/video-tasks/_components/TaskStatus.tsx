'use client'
import Icon from '@muse/ui/icons'
import { cn } from '@/lib/utils'
import { TaskStatus, getTaskStatus } from './utils'
import type { VideoWithTasksResult } from '@/lib/bindings'
import { Tooltip } from '@muse/ui/v2/tooltip'
import { forwardRef, HTMLAttributes, useMemo } from 'react'

export const VIDEO_DIMENSION: Record<string, [string, number]> = {
  // 任务类型: [任务名称, 任务排序, 完成以后是否显示]
  Frame: ['帧处理', 1],
  FrameContentEmbedding: ['画面索引', 2],
  FrameCaption: ['视频描述', 3],
  FrameCaptionEmbedding: ['描述索引', 4],
  Audio: ['音频提取', 5],
  Transcript: ['语音转译', 6],
  TranscriptEmbedding: ['语音索引', 7],
}

type Props = {
  status: TaskStatus
  name: string
} & HTMLAttributes<HTMLDivElement>

function TaskItemStatus({ status, name, className }: Props) {
  const statusIcon = useMemo(() => {
    switch (status) {
      case TaskStatus.Failed:
        return <Icon.Close className='w-4 h-4 text-red-500' />
      case TaskStatus.Cancelled:
        return <Icon.Close className='w-4 h-4 text-neutral-900' />
      case TaskStatus.Done:
        return <Icon.Check className='w-4 h-4 text-green-500' />
      case TaskStatus.Processing:
        return <Icon.Cycle className='w-4 h-4 text-orange-500 animate-spin' />
      default:
        return <Icon.Clock className='w-4 h-4 text-neutral-900' />
    }
  }, [status])

  return (
    <div className={cn('flex gap-1 items-center min-w-24 justify-start py-1', className)}>
      <span>{statusIcon}</span>
      <div>{name}</div>
    </div>
  )
}

export function VideoTaskStatus({ tasks }: {
  tasks: VideoWithTasksResult['tasks']
}) {
  type GroupProps = {
    name: string,
    tasks: {
      name: string,
      index: number,
      status: TaskStatus,
    }[],
    status?: 'done' | 'processing',
  }

  const tasksGroups = useMemo<GroupProps[]>(() => {
    const map: Record<typeof tasks[number]['taskType'], GroupProps["tasks"][number]> = {}
    tasks.forEach((task) => {
      const [name, index] = VIDEO_DIMENSION[task.taskType] ?? []
      const status = getTaskStatus(task)
      map[task.taskType] = { name, index, status }
    })
    const result: GroupProps[] = [{
      name: 'Description',
      tasks: [
        map['Frame'],
        map['FrameContentEmbedding'],
        map['FrameCaption'],
        map['FrameCaptionEmbedding'],
      ],
    }, {
      name: 'Transcript',
      tasks: [
        map['Audio'],
        map['Transcript'],
        map['TranscriptEmbedding'],
      ],
    }]
    result.forEach((group) => {
      group.tasks = group.tasks.filter(Boolean)
      group.tasks.sort((a, b) => a.index - b.index)
      if (group.tasks.some((task) => task.status === TaskStatus.Processing)) {
        group.status = 'processing'
      }
      if (group.tasks.every((task) => task.status === TaskStatus.Done)) {
        group.status = 'done'
      }
    })
    return result
  }, [tasks])

  const GroupBadge = forwardRef<HTMLDivElement, { group: GroupProps}>(function XXX({ group, ...props }, forwardRef) {
    const [className, icon] = useMemo(() => {
      switch (group.status) {
        case 'done':
          return [
              "text-green-600 border-green-200 bg-green-100",
              <Icon.Check key={group.name} className='w-3 h-3' />
          ]
        case 'processing':
          return [
              "text-orange-600 border-orange-200 bg-orange-100",
              <Icon.Cycle key={group.name} className='w-3 h-3 animate-spin' />
          ]
        default:
          return [
              "text-neutral-600 border-neutral-200 bg-neutral-100",
              <Icon.Clock key={group.name} className='w-3 h-3' />
          ]
      }
    }, [group])

    return (
      <div ref={forwardRef} {...props} className={cn(
        'flex gap-1 items-center justify-center rounded-full mr-2',
        'border border-app-line px-3 text-xs',
        className,
      )}>
        {icon}
        {group.name}
      </div>
    )
  })

  return (
    <div className="flex gap-2 items-center justify-end">
      {tasksGroups.map((group, _i) => (
        <Tooltip.Provider delayDuration={200} key={_i}>
          <Tooltip.Root>
            <Tooltip.Trigger asChild>
              <GroupBadge group={group} />
            </Tooltip.Trigger>
            <Tooltip.Portal>
              <Tooltip.Content>
                {group.tasks.map(({ name, index, status }) => (
                  <TaskItemStatus key={index} name={name} status={status} />
                ))}
              </Tooltip.Content>
            </Tooltip.Portal>
          </Tooltip.Root>
        </Tooltip.Provider>
      ))}
    </div>
  )
}
