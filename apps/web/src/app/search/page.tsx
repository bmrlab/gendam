'use client'
import PageNav from '@/components/PageNav'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import Viewport from '@/components/Viewport'
import type { SearchRequestPayload, SearchResultPayload } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import classNames from 'classnames'
import { useSearchParams } from 'next/navigation'
import { useCallback, useMemo, useState } from 'react'
import SearchForm from './SearchForm'
import VideoItem from './VideoItem'

const useSearchPayloadInURL: () => [
  SearchRequestPayload | null,
  (payload: SearchRequestPayload) => void,
] = () => {
  const searchParams = useSearchParams()
  const searchPayloadInURL = useMemo<SearchRequestPayload | null>(() => {
    try {
      const text = searchParams.get('text')
      const recordType = searchParams.get('recordType')
      if (text && recordType) {
        return { text, recordType }
      }
    } catch (e) {}
    return null
  }, [searchParams])

  const updateSearchPayloadInURL = useCallback((payload: SearchRequestPayload) => {
    const search = new URLSearchParams()
    search.set('text', payload.text)
    search.set('recordType', payload.recordType)
    window.history.replaceState({}, '', `${window.location.pathname}?${search}`)
  }, [])

  return [searchPayloadInURL, updateSearchPayloadInURL]
}

export default function Search() {
  const [searchPayloadInURL, updateSearchPayloadInURL] = useSearchPayloadInURL()

  const [searchPayload, setSearchPayload] = useState<SearchRequestPayload | null>(searchPayloadInURL)
  const queryRes = rspc.useQuery(['video.search.all', searchPayload!], {
    enabled: !!searchPayload,
  })

  const quickViewStore = useQuickViewStore()

  const handleVideoClick = useCallback(
    (item: SearchResultPayload) => {
      quickViewStore.open({
        name: item.name,
        assetObject: {
          id: item.assetObjectId,
          hash: item.assetObjectHash,
        },
        video: {
          currentTime: item.startTime / 1e3,
        },
      })
    },
    [quickViewStore],
  )

  const handleSearch = useCallback(
    (text: string, recordType: string) => {
      if (text && recordType) {
        const payload = { text, recordType }
        setSearchPayload(payload)
        updateSearchPayloadInURL(payload)
      }
    },
    [setSearchPayload, updateSearchPayloadInURL],
  )

  return (
    <Viewport.Page>
      <Viewport.Toolbar className="justify-center">
        <PageNav title="搜索" />
        <div className="mr-auto"></div>
        <SearchForm
          initialSearchPayload={searchPayloadInURL}
          onSubmit={(text: string, recordType: string) => handleSearch(text, recordType)}
        />
        <div className="ml-auto"></div>
      </Viewport.Toolbar>
      <Viewport.Content>
        {searchPayload ? (
          <div className="border-app-line flex items-center justify-start border-b px-8 py-2">
            <div className="border-app-line flex items-center overflow-hidden rounded-lg border text-xs">
              <div
                className={classNames('px-4 py-2', searchPayload.recordType === 'Frame' && 'bg-app-hover')}
                onClick={() => handleSearch(searchPayload.text, 'Frame')}
              >
                视频内容
              </div>
              <div
                className={classNames('px-4 py-2', searchPayload.recordType === 'Transcript' && 'bg-app-hover')}
                onClick={() => handleSearch(searchPayload.text, 'Transcript')}
              >
                视频语音
              </div>
            </div>
            <div className="text-ink/50 ml-4 text-sm">{searchPayload.text}</div>
          </div>
        ) : null}
        <div className="p-8">
          {queryRes.isLoading ? (
            <div className="text-ink/50 flex items-center justify-center px-2 py-8 text-sm">正在搜索...</div>
          ) : (
            <div className="flex flex-wrap gap-4">
              {queryRes.data?.map((item: SearchResultPayload, index: number) => {
                return (
                  <VideoItem
                    key={`${item.assetObjectId}-${index}`}
                    item={item}
                    handleVideoClick={handleVideoClick}
                  ></VideoItem>
                )
              })}
            </div>
          )}
        </div>
      </Viewport.Content>
    </Viewport.Page>
  )
}
