import { FilePath } from '@/lib/bindings'
import { queryClient, rspc } from '@/lib/rspc'
import { useCallback, useMemo } from 'react'

export const TaskItemType: Record<string, [string, number]> = {
  'frame': ['Visual Processing', 1],
  'frame-content-embedding': ['Visual Indexing', 2],
  'frame-caption': ['Description Recognition', 3],
  'frame-caption-embedding': ['Description Indexing', 4],
  'audio': ['Audio Processing', 5],
  'transcript': ['Speech Recognition', 6],
  'transcript-embedding': ['Transcript Indexing', 7],
}

export function useSortedTasks(data: FilePath['assetObject']) {
  const tasksQueryParams = useMemo(() => {
    const filter = { assetObjectId: data?.id }
    return { filter }
  }, [data?.id])
  const tasksQuery = rspc.useQuery(['tasks.list', tasksQueryParams], {
    enabled: !!data?.id,
  })
  const cancelJobsMut = rspc.useMutation(['video.tasks.cancel'])
  const handleJobsCancel = useCallback(async () => {
    if (!data?.id) {
      return
    }
    try {
      await cancelJobsMut.mutateAsync({
        assetObjectId: data.id,
        taskTypes: null,
      })
      queryClient.invalidateQueries({
        queryKey: ['tasks.list', tasksQueryParams],
      })
    } catch (error) {}
  }, [data?.id, cancelJobsMut, tasksQueryParams])

  const sortedTasks = useMemo(() => {
    if (!tasksQuery.data) {
      return []
    }
    return tasksQuery.data.sort((a, b) => {
      const [, indexA] = TaskItemType[a.taskType] ?? [, 0]
      const [, indexB] = TaskItemType[b.taskType] ?? [, 0]
      return indexA - indexB
    })
  }, [tasksQuery.data])

  return {
    sortedTasks,
    handleJobsCancel,
  }
}
