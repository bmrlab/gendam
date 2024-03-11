import type { VideoItem } from '@/app/video-tasks/_components/task-item'
import { MuseStatus } from '@/components/Badge'

export const hasProcessing = (tasks: VideoItem['tasks']) => {
  const dimension = Object.keys(VIDEO_DIMENSION)
  return tasks
    .filter((t) => dimension.includes(t.taskType))
    .map(getTaskStatus)
    .some((status) => status === MuseStatus.Processing)
}

export const getTaskStatus = (task: VideoItem['tasks'][number]) => {
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
  // FrameContentEmbedding: '图像特征',
}
