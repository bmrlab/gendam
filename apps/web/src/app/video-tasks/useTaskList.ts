import { TaskListRequestFilter, VideoWithTasksResult, TaskListRequestPayload } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { useBoundStore } from './_store'

export type TaskListProps = {
  pageSize: number,
  pageIndex: number,
  filter: TaskListRequestFilter
}

const validateProps = ({ pageSize, pageIndex, filter }: TaskListProps) => {
  pageSize = Math.max(10, parseInt(''+pageSize) || 10)
  pageIndex = Math.max(1, parseInt(''+pageIndex) || 1)
  if (filter !== 'all' && filter !== 'excludeCompleted') {
    filter = 'excludeCompleted'
  }
  return { pageSize, pageIndex, filter }
}

export default function useTaskList(props: TaskListProps = {
  pageSize: 10,
  pageIndex: 1,
  filter: 'excludeCompleted',
}) {
  props = validateProps(props)
  const pageSize = props.pageSize
  const [pageIndex, setPageIndex] = useState(props.pageIndex)
  const [filter, setFilter] = useState<TaskListRequestFilter>(props.filter)
  const setTaskListRefetch = useBoundStore.use.setTaskListRefetch()

  const { data, isLoading, refetch } = rspc.useQuery(
    [
      'video.tasks.list',
      {
        pagination: {
          pageIndex: pageIndex,
          pageSize: pageSize,
        },
        filter,
      },
    ],
    {
      refetchInterval: 5000,
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      refetchOnReconnect: false,
    },
  )

  useEffect(() => {
    setTaskListRefetch(refetch)
  }, [refetch, setTaskListRefetch])

  const [hasNextPage, maxPage] = useMemo(() => {
    const { maxPage } = data || { maxPage: 1 }
    return [pageIndex < maxPage, maxPage]
  }, [pageIndex, data])

  const fetchNextPage = useCallback(() => {
    if (hasNextPage) {
      setPageIndex((pageIndex) => pageIndex + 1)
    }
  }, [hasNextPage])

  const videos = useMemo(() => {
    return data ? data.data : ([] as VideoWithTasksResult[])
  }, [data])

  return {
    videos,
    maxPage,
    pageSize,
    pageIndex,
    setPageIndex,
    filter,
    setFilter,

    hasNextPage,
    fetchNextPage,

    isLoading,
    refetch,
  }
}
