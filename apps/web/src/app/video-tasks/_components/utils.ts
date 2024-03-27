import { MuseStatus } from '@/components/Badge'
import type { VideoWithTasksResult } from '@/lib/bindings'

export const hasProcessing = (tasks: VideoWithTasksResult['tasks']) => {
  const dimension = Object.keys(VIDEO_DIMENSION)
  return tasks
    .filter((t) => dimension.includes(t.taskType))
    .map(getTaskStatus)
    .some((status) => status === MuseStatus.Processing)
}

export const getTaskStatus = (task: VideoWithTasksResult['tasks'][number]) => {
  const exitCode = task.exitCode ?? -1
  if (exitCode > 1) {
    return MuseStatus.Failed
  }
  if (exitCode === 1) {
    return MuseStatus.Cancelled
  }
  if (exitCode === 0) {
    return MuseStatus.Done
  }
  if (task.startsAt) {
    return MuseStatus.Processing // 已经开始但还没结束
  }
  return MuseStatus.None
}

export const VIDEO_DIMENSION: Record<string, [string, number, boolean]> = {
  // 任务类型: [任务名称, 任务排序, 完成以后是否显示]
  Frame: ['帧处理', 1, false],
  FrameContentEmbedding: ['视频特征', 2, true],
  FrameCaption: ['视频描述', 3, false],
  FrameCaptionEmbedding: ['视频描述', 4, true],
  Audio: ['音频处理', 5, false],
  Transcript: ['视频语音', 6, true],
  TranscriptEmbedding: ['视频语音', 7, false],
}
