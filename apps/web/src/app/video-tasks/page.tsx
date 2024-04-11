'use client'
import Viewport from '@/components/Viewport'
import { type TaskListRequestFilter } from '@/lib/bindings'
import PageNav from '@/components/PageNav'
import { ScrollArea } from '@muse/ui/v1/scroll-area'
import { Checkbox } from '@muse/ui/v2/checkbox'
import classNames from 'classnames'
import { useSearchParams } from 'next/navigation'
import { useEffect, useMemo } from 'react'
import AudioDialog from './_components/audio/AudioDialog'
import TaskFooter from './_components/footer'
import VideoTasksList from './_components/TaskList'
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
      <form className='flex items-center gap-2 mr-3'>
        <Checkbox.Root
          id="task-filter" className=""
          checked={filter === 'all'}
          onCheckedChange={(checked) => setFilter(checked ? 'all' : 'excludeCompleted')}
        >
          <Checkbox.Indicator />
        </Checkbox.Root>
        <label className="text-xs" htmlFor="task-filter">
          Show completed tasks
        </label>
      </form>
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
      {/* <VideoTaskHeader /> */}
      <Viewport.Toolbar className="items-center justify-between">
        <PageNav title="任务列表" />
        <ListFilter />
      </Viewport.Toolbar>
      <Viewport.Content>
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
