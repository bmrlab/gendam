import type { VideoWithTasksResult } from '@/lib/bindings'
import { MuseStatus } from '@/components/Badge'

export const hasProcessing = (tasks: VideoWithTasksResult['tasks']) => {
  const dimension = Object.keys(VIDEO_DIMENSION)
  return tasks
    .filter((t) => dimension.includes(t.taskType))
    .map(getTaskStatus)
    .some((status) => status === MuseStatus.Processing)
}

export const getTaskStatus = (task: VideoWithTasksResult['tasks'][number]) => {
  if (task.startsAt && !task.endsAt) {
    return MuseStatus.Processing
  }
  if (task.startsAt && task.endsAt) {
    return MuseStatus.Done
  }
  return MuseStatus.Failed
}

export const VIDEO_DIMENSION: Record<string, string> = {
  // Audio: '语音转译',
  // "Transcript": "语音转译",
  TranscriptEmbedding: '语音转译',
  // "FrameCaption": "图像描述",
  FrameCaptionEmbedding: '图像描述',
  // "Frame": "图像特征",
  FrameContentEmbedding: '图像特征',
}
