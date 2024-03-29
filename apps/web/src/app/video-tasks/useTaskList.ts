import { Filter, VideoWithTasksResult } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { useEffect, useMemo, useState } from 'react'
import { useBoundStore } from './_store'

export type TaskListProps = {
  limit?: number
  filter?: Filter
}

const DEFAULT_LIMIT = 100

export default function useTaskList({ limit = DEFAULT_LIMIT }: TaskListProps) {
  const [data, setData] = useState<VideoWithTasksResult[][]>([])
  const [pageIndex, setPageIndex] = useState(0)
  const [maxPages, setMaxPages] = useState<number>(0)
  const setTaskListRefetch = useBoundStore.use.setTaskListRefetch()
  const filter = useBoundStore.use.taskFilter()

  // 拿分页数据
  const { data: videos } = rspc.useQuery(
    [
      'video.tasks.list',
      {
        pagination: {
          pageIndex: pageIndex,
          pageSize: limit,
        },
        filter,
      },
    ],
    {
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      refetchOnReconnect: false,
    },
  )

  // 拿全量数据
  const { data: fullVideo, refetch } = rspc.useQuery(
    [
      'video.tasks.list',
      {
        pagination: {
          pageIndex: 0,
          pageSize: limit * (pageIndex + 1),
        },
        filter,
      },
    ],
    {
      // 不会触发请求，除非手动调用 refetch
      enabled: false,
      refetchOnWindowFocus: false,
      refetchOnMount: false,
      refetchOnReconnect: false,
    },
  )

  useEffect(() => {
    setPageIndex(0)
    refetch()
  }, [filter, refetch, setPageIndex])

  // 使用全量数据拆分成多页
  useEffect(() => {
    if (fullVideo && fullVideo.data.length > 0) {
      let newData = new Array(Math.ceil(fullVideo.data.length / limit))
        .fill(null)
        .map((_, i) => fullVideo.data.slice(i * limit, (i + 1) * limit))
      setData(newData)
    }
  }, [fullVideo, limit])

  const hasNextPage = useMemo(() => pageIndex < maxPages, [pageIndex, maxPages])

  // 拿到分页数据后，更新对应页的数据
  useEffect(() => {
    if (videos) {
      setMaxPages(videos.maxPage)
      let newData = [...data]
      newData[pageIndex] = videos.data
      setData(newData)
    }
  }, [pageIndex, videos])

  const fetchNextPage = () => {
    if (hasNextPage) {
      setPageIndex((pageIndex) => pageIndex + 1)
    }
  }

  useEffect(() => {
    setTaskListRefetch(refetch)
  }, [refetch, setTaskListRefetch])

  return {
    data: data.flat(),
    hasNextPage,
    fetchNextPage,
    isLoading: data.length === 0 && !videos,
    refetch,
  }
}
