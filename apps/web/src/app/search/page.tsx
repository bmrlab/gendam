'use client'
import PageNav from '@/components/PageNav'
import Viewport from '@/components/Viewport'
import type { SearchRequestPayload, SearchResultPayload } from '@/lib/bindings'
import { Video_Files } from '@muse/assets/images'
import Image from 'next/image'
import { rspc } from '@/lib/rspc'
import classNames from 'classnames'
import { useSearchParams } from 'next/navigation'
import React, { useCallback, useMemo, useState } from 'react'
import SearchForm from './SearchForm'
import VideoItem from './VideoItem'
import { Checkbox } from '@muse/ui/v2/checkbox'

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
  const [groupFrames, setGroupFrames] = useState(false)

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
      <Viewport.Toolbar className="justify-start">
        <PageNav title="Search" className="w-1/3" />
        <div className="w-1/3">
          <SearchForm
            initialSearchPayload={searchPayloadInURL}
            onSubmit={(text: string, recordType: string) => handleSearch(text, recordType)}
          />
        </div>
        <div className="ml-auto"></div>
      </Viewport.Toolbar>
      <Viewport.Content className="overflow-hidden flex flex-col items-stretch">
        {searchPayload ? (
          <div className="border-app-line flex items-center justify-start border-b px-8 py-2">
            <div className="border-app-line flex items-center overflow-hidden rounded-lg border text-xs">
              <div
                className={classNames('px-4 py-2', searchPayload.recordType === 'Frame' && 'bg-app-hover')}
                onClick={() => handleSearch(searchPayload.text, 'Frame')}
              >Visual</div>
              <div
                className={classNames('px-4 py-2', searchPayload.recordType === 'Transcript' && 'bg-app-hover')}
                onClick={() => handleSearch(searchPayload.text, 'Transcript')}
              >Transcript</div>
            </div>
            <div className="text-ink/50 ml-4 text-sm flex-1 truncate">{searchPayload.text}</div>
            <form className='flex items-center gap-2 mr-3'>
            <Checkbox.Root
              id="--group-frames" checked={groupFrames}
              onCheckedChange={(checked: boolean | 'indeterminate') => {
                setGroupFrames(checked === true ? true : false)
              }}
            >
              <Checkbox.Indicator />
            </Checkbox.Root>
            <label className="text-xs" htmlFor="--group-frames">Expand video frames</label>
          </form>
          </div>
        ) : null}
        <div className="flex-1 overflow-auto p-8">
          {!searchPayload ? (
            <div className="h-full flex flex-col items-center justify-center">
              <Image src={Video_Files} alt="video files" priority className="w-60 h-60"></Image>
              <div className="my-4 text-sm">Search for visual objects or processed transcripts</div>
            </div>
          ) : queryRes.isLoading ? (
            <div className="text-ink/50 flex items-center justify-center px-2 py-8 text-sm">Searching...</div>
          ) : queryRes.isSuccess && queryRes.data.length === 0 ? (
            <div className="h-full flex flex-col items-center justify-center">
              <Image src={Video_Files} alt="video files" priority className="w-60 h-60"></Image>
              <div className="my-4 text-sm">
                No results found for <span className="font-medium">{searchPayload.text}</span>
              </div>
            </div>
          ) : queryRes.isSuccess && queryRes.data.length > 0 ? (
            <div className="min-h-full pb-8 flex flex-wrap gap-4 content-start">
              {queryRes.data.map((item: SearchResultPayload, index: number) => (
                <VideoItem
                  key={`${item.assetObjectId}-${index}`}
                  item={item} groupFrames={groupFrames}
                ></VideoItem>
              ))}
            </div>
          ) : (
            <div>Something went wrong</div>
          )}
        </div>
      </Viewport.Content>
    </Viewport.Page>
  )
}
