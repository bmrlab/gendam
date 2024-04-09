'use client'
import Icon from '@/components/Icon'
import PageNav from '@/components/PageNav'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import Viewport from '@/components/Viewport'
import type { SearchResultPayload } from '@/lib/bindings'
import { SearchRequestPayload } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { formatDuration } from '@/lib/utils'
import classNames from 'classnames'
import Image from 'next/image'
import { useSearchParams } from 'next/navigation'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

const VideoItem: React.FC<{
  item: SearchResultPayload
  handleVideoClick: (item: SearchResultPayload) => void
}> = ({ item, handleVideoClick }) => {
  const currentLibrary = useCurrentLibrary()
  const videoRef = useRef<HTMLVideoElement>(null)

  useEffect(() => {
    const video = videoRef.current
    if (!video) return
    let startTime = Math.max(0, item.startTime / 1e3 - 0.5)
    let endTime = startTime + 2
    video.currentTime = startTime
    video.ontimeupdate = () => {
      if (video.currentTime >= endTime) {
        // video.pause();
        // video.ontimeupdate = null;
        video.currentTime = startTime
      }
    }
  }, [item])

  return (
    <div
      className="invisible relative w-64 overflow-hidden rounded-md shadow-md hover:visible"
      onClick={() => handleVideoClick(item)}
    >
      <div className="visible relative h-36 w-full cursor-pointer bg-neutral-100">
        {/* <video
          ref={videoRef}
          controls={false}
          autoPlay={false}
          muted
          loop
          style={{ width: '100%', height: '100%', objectFit: 'cover' }}
        >
          <source src={currentLibrary.getFileSrc(item.assetObjectHash)} />
        </video> */}
        <Image
          src={currentLibrary.getThumbnailSrc(item.assetObjectHash, Math.floor(item.startTime / 1e3))}
          alt={item.name}
          fill={true}
          className="object-cover"
          priority
        ></Image>
      </div>
      <div className="absolute left-0 top-0 flex h-full w-full flex-col justify-between bg-black/60 px-4 py-2 text-neutral-300">
        <div className="truncate text-xs">
          {item.materializedPath}
          {item.name}
        </div>
        <div className="truncate text-xs">{formatDuration(item.startTime / 1000)}</div>
      </div>
    </div>
  )
}

export default function Search() {
  const searchParams = useSearchParams()
  const searchPayloadInSearchParams = useMemo<SearchRequestPayload | null>(() => {
    try {
      const q = searchParams.get('q')
      if (q) {
        return JSON.parse(q)
      } else {
        return null
      }
    } catch (e) {
      return null
    }
  }, [searchParams])

  const [searchPayload, setSearchPayload] = useState<SearchRequestPayload | null>(searchPayloadInSearchParams)
  const queryRes = rspc.useQuery(['video.search.all', searchPayload!], {
    enabled: !!searchPayload,
  })

  const quickViewStore = useQuickViewStore()
  const searchInputRef = useRef<HTMLInputElement>(null)

  const [keywordTyping, setKeywordTyping] = useState<string>('')

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
    (text: string, recordType: string = 'Frame') => {
      if (text && recordType) {
        const payload = { text, recordType }
        setSearchPayload(payload)
        const search = new URLSearchParams()
        search.set('q', JSON.stringify(payload))
        window.history.replaceState({}, '', `${window.location.pathname}?${search}`)
      }
    },
    [setSearchPayload],
  )

  const [focused, setFocused] = useState(false)

  return (
    <Viewport.Page>
      <Viewport.Toolbar className="justify-center">
        <PageNav title="搜索" />
        <div className="mr-auto"></div>
        <div className="relative w-80">
          <form
            onSubmit={(e) => {
              e.preventDefault()
              handleSearch(keywordTyping)
              if (searchInputRef.current) {
                searchInputRef.current.blur()
                searchInputRef.current.value = ''
              }
            }}
            className="relative block"
          >
            <input
              ref={searchInputRef}
              type="text"
              className="block w-full rounded-md border border-app-line bg-app-overlay px-4 py-[0.3rem] pl-7 text-sm text-ink outline-none"
              placeholder="搜索"
              onInput={(e) => setKeywordTyping(e.currentTarget.value)}
              onFocus={(e) => setFocused(true)}
              onBlur={(e) => setTimeout(() => setFocused(false), 200)}
            />
            <Icon.search className="absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 transform text-ink/50" />
          </form>
          {focused && keywordTyping ? (
            <div className="absolute top-full z-10 w-full rounded-md border border-app-line bg-app-box p-1 text-sm shadow-md">
              <div className="px-2 py-1 text-ink/50">搜索类型</div>
              <div
                className="flex items-center justify-start rounded-md px-2 py-2 text-ink hover:bg-app-hover"
                onClick={() => handleSearch(keywordTyping, 'Frame')}
              >
                <span className="text-ink/50">
                  <Icon.image className="w-4" />
                </span>
                <span className="mx-2">搜索视频内容</span>
                <strong>{keywordTyping}</strong>
              </div>
              <div
                className="flex items-center justify-start rounded-md px-2 py-2 text-ink hover:bg-app-hover"
                onClick={() => handleSearch(keywordTyping, 'Transcript')}
              >
                <span className="text-ink/50">
                  <Icon.microphone className="w-4" />
                </span>
                <span className="mx-2">搜索视频语音</span>
                <strong>{keywordTyping}</strong>
              </div>
            </div>
          ) : null}
        </div>
        <div className="ml-auto"></div>
      </Viewport.Toolbar>
      <Viewport.Content>
        {searchPayload ? (
          <div className="flex items-center justify-start border-b border-app-line px-8 py-2">
            <div className="flex items-center overflow-hidden rounded-lg border border-app-line text-xs">
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
            <div className="ml-4 text-sm text-ink/50">{searchPayload.text}</div>
          </div>
        ) : null}
        <div className="p-8">
          {queryRes.isLoading ? (
            <div className="flex items-center justify-center px-2 py-8 text-sm text-ink/50">正在搜索...</div>
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
