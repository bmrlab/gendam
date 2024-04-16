import type { VideoWithTasksResult } from '@/lib/bindings'

// export const hasProcessing = (tasks: VideoWithTasksResult['tasks']) => {
//   const dimension = Object.keys(VIDEO_DIMENSION)
//   return tasks
//     .filter((t) => dimension.includes(t.taskType))
//     .map(getTaskStatus)
//     .some((status) => status === TaskStatus.Processing)
// }

export const hasAudio = (tasks: VideoWithTasksResult['tasks']) => {
  return tasks.some((task) => task.taskType === 'Audio' && task.exitCode === 0)
}

export enum TaskStatus {
  Failed,
  Cancelled,
  Done,
  Processing,
  None,
}

export const getTaskStatus = (task: VideoWithTasksResult['tasks'][number]) => {
  const exitCode = task.exitCode ?? -1
  if (exitCode > 1) {
    return TaskStatus.Failed
  }
  if (exitCode === 1) {
    return TaskStatus.Cancelled
  }
  if (exitCode === 0) {
    return TaskStatus.Done
  }
  if (task.startsAt) {
    return TaskStatus.Processing // 已经开始但还没结束
  }
  return TaskStatus.None
}
