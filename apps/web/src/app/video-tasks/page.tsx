'use client'
import Viewport from '@/components/Viewport'
import { type VideoTaskListRequestFilter } from '@/lib/bindings'
import { Drop_To_Folder } from '@gendam/assets/images'
import Image from 'next/image'
import Link from 'next/link'
import PageNav from '@/components/PageNav'
import { ScrollArea } from '@gendam/ui/v1/scroll-area'
import { Checkbox } from '@gendam/ui/v2/checkbox'
import classNames from 'classnames'
import { useSearchParams } from 'next/navigation'
import { useCallback, useEffect, useMemo } from 'react'
import TaskFooter from './_components/footer'
import VideoTasksList from './_components/TaskList'
import { useBoundStore } from './_store'
import useTaskList, { type TaskListProps } from './useTaskList'

type SearchPayloadInURL = {
  filter: string,
  pageIndex: string,
  pageSize: string,
}

const useSearchPayloadInURL: () => [
  SearchPayloadInURL | null,
  (payload: TaskListProps) => void,
] = () => {
  const searchParams = useSearchParams()
  const searchPayloadInURL = useMemo<SearchPayloadInURL | null>(() => {
    try {
      const filter = searchParams.get('filter')
      const pageIndex = searchParams.get('pageIndex')
      const pageSize = searchParams.get('pageSize')
      if (filter && pageIndex && pageSize) {
        return { filter, pageIndex, pageSize }
      }
    } catch (e) {}
    return null
  }, [searchParams])

  const updateSearchPayloadInURL = useCallback((payload: TaskListProps) => {
    const search = new URLSearchParams()
    search.set('filter', payload.filter.toString())
    search.set('pageIndex', payload.pageIndex.toString())
    search.set('pageSize', payload.pageSize.toString())
    window.history.replaceState({}, '', `${window.location.pathname}?${search}`)
  }, [])

  return [searchPayloadInURL, updateSearchPayloadInURL]
}

function validateSearchPayload(searchPayloadInURL: SearchPayloadInURL | null): TaskListProps {
  if (!searchPayloadInURL) {
    return { pageSize: 10, pageIndex: 1, filter: 'all'}
  }
  const pageSize = Math.max(10, parseInt(''+searchPayloadInURL.pageSize) || 10)
  const pageIndex = Math.max(1, parseInt(''+searchPayloadInURL.pageIndex) || 1)
  let filter = 'all' as VideoTaskListRequestFilter
  if (searchPayloadInURL.filter === 'all' || searchPayloadInURL.filter === 'excludeCompleted') {
    filter = searchPayloadInURL.filter
  }
  return { pageSize, pageIndex, filter }
}

export default function VideoTasksPage() {
  const clearVideoSelected = useBoundStore.use.clearVideoSelected()
  const [searchPayloadInURL, updateSearchPayloadInURL] = useSearchPayloadInURL()

  const {
    videos,
    maxPage, pageSize, pageIndex, setPageIndex, filter, setFilter,
    // hasNextPage, fetchNextPage,
  } = useTaskList(validateSearchPayload(searchPayloadInURL))

  useEffect(() => {
    const payload: TaskListProps = { pageSize, pageIndex, filter }
    const search = new URLSearchParams()
    updateSearchPayloadInURL(payload)
  }, [filter, pageIndex, pageSize, updateSearchPayloadInURL])

  const ListFilter = () => {
    return (
      <form className='flex items-center gap-2 mr-3'>
        <Checkbox.Root
          id="--show-completed-tasks" className=""
          checked={filter === 'all'}
          onCheckedChange={(checked) => {
            setFilter(checked ? 'all' : 'excludeCompleted')
            setPageIndex(1)
          }}
        >
          <Checkbox.Indicator />
        </Checkbox.Root>
        <label className="text-xs" htmlFor="--show-completed-tasks">Show completed tasks</label>
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
        <PageNav title="All jobs" />
        <ListFilter />
      </Viewport.Toolbar>
      <Viewport.Content onClick={() => clearVideoSelected()}>
        {videos.length === 0 ? (
          <div className="h-full flex flex-col items-center justify-center">
            <Link href="/explorer" className="cursor-default">
              <Image src={Drop_To_Folder} alt="drop to folder" priority className="w-60 h-60"></Image>
              <div className="my-4 text-sm">Go to the <span className="underline">explorer</span> to add videos</div>
            </Link>
          </div>
        ) : (
          <ScrollArea className="flex-1 rounded-[6px]">
            <VideoTasksList data={videos} />
            {maxPage > 1 ? <Pagination /> : null}
          </ScrollArea>
        )}
      </Viewport.Content>
      <TaskFooter total={videos.length} />
    </Viewport.Page>
  )
}
