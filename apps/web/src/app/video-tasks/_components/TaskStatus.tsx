'use client'
import type { VideoWithTasksResult } from '@/lib/bindings'
import { cn } from '@/lib/utils'
import Icon from '@muse/ui/icons'
import { Tooltip } from '@muse/ui/v2/tooltip'
import { HTMLAttributes, forwardRef, useMemo } from 'react'
import { TaskStatus, getTaskStatus } from './utils'

export const VIDEO_DIMENSION: Record<string, [string, number]> = {
  // 任务类型: [任务名称, 任务排序, 完成以后是否显示]
  Frame: ['Frame Processing', 1],                      // 帧处理
  FrameContentEmbedding: ['Visual Indexing', 2],       // 画面索引
  FrameCaption: ['Video Recognition', 3],              // 视频描述
  FrameCaptionEmbedding: ['Description Indexing', 4],  // 描述索引
  Audio: ['Audio Processing', 5],                      // 音频提取
  Transcript: ['Speech Recognition', 6],               // 语音转译
  TranscriptEmbedding: ['Transcript Indexing', 7],     // 语音索引
}

type Props = {
  status: TaskStatus
  name: string
} & HTMLAttributes<HTMLDivElement>

function TaskItemStatus({ status, name, className }: Props) {
  const statusIcon = useMemo(() => {
    switch (status) {
      case TaskStatus.Failed:
        return <Icon.Close className="h-4 w-4 text-red-500" />
      case TaskStatus.Cancelled:
        return <Icon.Close className="h-3 w-3 text-neutral-800" />
      case TaskStatus.Done:
        return <Icon.Check className="h-4 w-4 text-green-500" />
      case TaskStatus.Processing:
        return <Icon.Cycle className="h-4 w-4 animate-spin text-orange-500" />
      default:
        return <Icon.Clock className="h-4 w-4 text-neutral-900" />
    }
  }, [status])

  return (
    <div className={cn('flex min-w-24 items-center justify-start gap-1 py-1', className)}>
      <span>{statusIcon}</span>
      <div>{name}</div>
    </div>
  )
}

export function VideoTaskStatus({ tasks }: { tasks: VideoWithTasksResult['tasks'] }) {
  type GroupProps = {
    name: string
    tasks: {
      name: string
      index: number
      status: TaskStatus
    }[]
    status?: 'done' | 'processing'
  }

  const tasksGroups = useMemo<GroupProps[]>(() => {
    const map: Record<(typeof tasks)[number]['taskType'], GroupProps['tasks'][number]> = {}
    tasks.forEach((task) => {
      const [name, index] = VIDEO_DIMENSION[task.taskType] ?? []
      const status = getTaskStatus(task)
      map[task.taskType] = { name, index, status }
    })
    const result: GroupProps[] = [
      {
        name: 'Description',
        tasks: [map['Frame'], map['FrameContentEmbedding'], map['FrameCaption'], map['FrameCaptionEmbedding']],
      },
      {
        name: 'Transcript',
        tasks: [map['Audio'], map['Transcript'], map['TranscriptEmbedding']],
      },
    ]
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

  const GroupBadge = forwardRef<HTMLDivElement, { group: GroupProps }>(function XXX({ group, ...props }, forwardRef) {
    const [className, icon] = useMemo(() => {
      switch (group.status) {
        case 'done':
          return ['text-green-600 border-green-200 bg-green-100', <Icon.Check key={group.name} className="h-3 w-3" />]
        case 'processing':
          return [
            'text-orange-600 border-orange-200 bg-orange-100',
            <Icon.Cycle key={group.name} className="h-3 w-3 animate-spin" />,
          ]
        default:
          return [
            'text-neutral-600 border-neutral-200 bg-neutral-100',
            <Icon.Clock key={group.name} className="h-3 w-3" />,
          ]
      }
    }, [group])

    return (
      <div
        ref={forwardRef}
        {...props}
        className={cn(
          'mr-2 flex items-center justify-center gap-1 rounded-full',
          'border-app-line border px-3 text-xs',
          className,
        )}
      >
        {icon}
        {group.name}
      </div>
    )
  })

  return (
    <div className="flex items-center justify-end gap-2">
      {tasksGroups.map(
        (group, _i) =>
          group.tasks.length > 0 && (
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
          ),
      )}
    </div>
  )
}
