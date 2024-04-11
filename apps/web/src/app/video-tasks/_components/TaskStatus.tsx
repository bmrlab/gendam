'use client'
import Icon from '@muse/ui/icons'
import { cn } from '@/lib/utils'
import { TaskStatus, getTaskStatus } from './utils'
import type { VideoWithTasksResult } from '@/lib/bindings'
import { Tooltip } from '@muse/ui/v2/tooltip'
import { HTMLAttributes, useMemo } from 'react'

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
    done: boolean,
  }

  const tasksGroup = useMemo<GroupProps[]>(() => {
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
      done: false
    }, {
      name: 'Transcript',
      tasks: [
        map['Audio'],
        map['Transcript'],
        map['TranscriptEmbedding'],
      ],
      done: false
    }]
    result.forEach((group) => {
      group.tasks = group.tasks.filter(Boolean)
      group.tasks.sort((a, b) => a.index - b.index)
      group.done = group.tasks.every((task) => task.status === TaskStatus.Done)
    })
    return result
  }, [tasks])

  return (
    <div className="flex gap-2 items-center justify-end">
      {tasksGroup.map(({ name, tasks, done }, _i) => (
        <Tooltip.Provider delayDuration={200} key={_i}>
          <Tooltip.Root>
            <Tooltip.Trigger asChild>
              <div className={cn(
                'flex gap-1 items-center justify-center rounded-full mr-2',
                'border border-app-line px-3 text-xs text-neutral-600',
                done ? 'border-green-200 bg-green-100' : 'border-orange-200 bg-orange-100'
              )}>
                {done ? <Icon.Check className='w-3 h-3' /> : <Icon.Clock className='w-3 h-3' />}
                {name}
              </div>
            </Tooltip.Trigger>
            <Tooltip.Portal>
              <Tooltip.Content>
                {tasks.map(({ name, index, status }) => (
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
