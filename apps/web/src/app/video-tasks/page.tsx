'use client'
import Viewport from '@/components/Viewport'
import { TaskListRequestFilter } from '@/lib/bindings'
import { Label } from '@muse/ui/v1/label'
import { RadioGroup, RadioGroupItem } from '@muse/ui/v1/radio-group'
import { ScrollArea } from '@muse/ui/v1/scroll-area'
import classNames from 'classnames'
import { useSearchParams } from 'next/navigation'
import { useEffect, useMemo } from 'react'
import AudioDialog from './_components/audio/AudioDialog'
import TaskFooter from './_components/footer'
import VideoTaskHeader from './_components/header'
import VideoTasksList from './_components/task-list'
import useTaskList, { type TaskListProps } from './useTaskList'

export default function VideoTasksPage() {
  const searchParams = useSearchParams()
  const filterInSearchParams = useMemo<TaskListProps | undefined>(() => {
    try {
      const q = searchParams.get('q')
      if (q) {
        return JSON.parse(q)
      } else {
        return undefined
      }
    } catch (e) {
      return undefined
    }
  }, [searchParams])

  const {
    videos, isLoading,
    maxPage, pageSize, pageIndex, setPageIndex, filter, setFilter,
    // hasNextPage, fetchNextPage,
  } = useTaskList(filterInSearchParams)

  useEffect(() => {
    const payload: TaskListProps = { pageSize, pageIndex, filter }
    const search = new URLSearchParams()
    search.set('q', JSON.stringify(payload))
    window.history.replaceState({}, '', `${window.location.pathname}?${search}`)
  }, [filter, pageIndex, pageSize])

  const ListFilter = () => {
    return (
      <RadioGroup
        defaultValue={filter as string}
        className="flex px-8 py-2"
        value={filter as string}
        onValueChange={(filter) => setFilter(filter as TaskListRequestFilter)}
      >
        <div className="flex items-center space-x-2">
          <RadioGroupItem value="all" id="task-filter-all" />
          <Label htmlFor="task-filter-all">全部</Label>
        </div>
        <div className="flex items-center space-x-2">
          <RadioGroupItem value="excludeCompleted" id="task-filter-excludeCompleted" />
          <Label htmlFor="task-filter-excludeCompleted">未完成</Label>
        </div>
      </RadioGroup>
    )
  }

  const Pagination = () => {
    return (
      <div className="my-3 flex items-center justify-center gap-1 p-3">
        {Array.from({ length: maxPage }).map((_, index) => {
          return (
            <div
              key={index + 1}
              className={classNames(
                'w-6 border border-app-line p-1 text-center text-xs leading-4',
                pageIndex === index + 1 ? 'bg-app-hover' : null,
              )}
              onClick={() => {
                if (pageIndex !== index + 1) setPageIndex(index + 1)
              }}
            >
              {index + 1}
            </div>
          )
        })}
      </div>
    )
  }

  return (
    <Viewport.Page>
      <VideoTaskHeader />
      <Viewport.Content>
        <ListFilter />
        <ScrollArea className="flex-1 rounded-[6px]">
          <VideoTasksList data={videos} isLoading={isLoading} />
          {maxPage > 1 ? <Pagination /> : null}
        </ScrollArea>
      </Viewport.Content>
      <TaskFooter total={videos.length} />
      <AudioDialog />
    </Viewport.Page>
  )
}

// const bottomRef = useRef(null)
// useEffect(() => {
//   // 防止初次加载
//   const timer = setTimeout(() => {
//     const observer = new IntersectionObserver(
//       ([entry]) => {
//         if (entry.isIntersecting) {
//           if (hasNextPage) {
//             fetchNextPage()
//           }
//         }
//       },
//       {
//         root: null,
//         threshold: 0.1,
//       },
//     )
//     if (bottomRef.current) {
//       observer.observe(bottomRef.current)
//     }
//   }, 500)

//   return () => {
//     clearTimeout(timer)
//   }
// }, [fetchNextPage, hasNextPage])

// <div className="flex items-center justify-center p-4">
//   {hasNextPage ? (
//     <>
//       <div className="text-xs text-slate-400">滚动加载更多</div>
//       <div ref={bottomRef}></div>
//     </>
//   ) : (
//     <div className="text-xs text-slate-400">没有更多了</div>
//   )}
// </div>
