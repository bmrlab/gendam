import { TaskStatus } from './TaskStatus'
import type { VideoWithTasksResult } from '@/lib/bindings'

// export const hasProcessing = (tasks: VideoWithTasksResult['tasks']) => {
//   const dimension = Object.keys(VIDEO_DIMENSION)
//   return tasks
//     .filter((t) => dimension.includes(t.taskType))
//     .map(getTaskStatus)
//     .some((status) => status === TaskStatus.Processing)
// }

export const isNotDone = (tasks: VideoWithTasksResult['tasks']) => {
  // 未开始和正在进行的
  return !!tasks.find((task) => getTaskStatus(task) !== TaskStatus.Done)
}

export const hasAudio = (tasks: VideoWithTasksResult['tasks']) => {
  return tasks.some((task) => task.taskType === 'Audio' && task.exitCode === 0)
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
