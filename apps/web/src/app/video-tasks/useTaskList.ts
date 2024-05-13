import type { VideoTaskListRequestFilter, VideoWithTasksResult, VideoTaskListRequestPayload } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { useBoundStore } from './_store'

export type TaskListProps = {
  pageSize: number,
  pageIndex: number,
  filter: VideoTaskListRequestFilter
}

export default function useTaskList(props: TaskListProps) {
  const pageSize = props.pageSize
  const [pageIndex, setPageIndex] = useState(props.pageIndex)
  const [filter, setFilter] = useState<VideoTaskListRequestFilter>(props.filter)
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
      // refetchOnWindowFocus: false,
      refetchOnMount: true,
      // refetchOnReconnect: false,
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

    // isLoading,
    refetch,
  }
}
