'use client'
import type { SearchResultPayload } from '@/lib/bindings'
import Icon from '@/components/Icon'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { SearchRequestPayload } from '@/lib/bindings'
import classNames from 'classnames'
import { formatDuration } from '@/lib/utils'

const VideoPreview: React.FC<{ item: SearchResultPayload }> = ({ item }) => {
  const currentLibrary = useCurrentLibrary()

  const videoRef = useRef<HTMLVideoElement>(null)
  let startTime = Math.max(0, item.startTime / 1e3 - 0.5)
  let endTime = startTime + 2

  useEffect(() => {
    const video = videoRef.current
    if (!video) return
    video.currentTime = startTime
    video.ontimeupdate = () => {
      if (video.currentTime >= endTime) {
        video.pause()
        video.ontimeupdate = null
      }
    }
  }, [startTime, endTime])

  return (
    <video ref={videoRef} controls autoPlay style={{ width: '100%', height: '100%', objectFit: 'contain' }}>
      <source src={currentLibrary.getFileSrc(item.assetObjectHash)} />
    </video>
  )
}

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
      className="w-64 relative overflow-hidden rounded-md shadow-md invisible hover:visible"
      onClick={() => handleVideoClick(item)}
    >
      <div className="relative h-36 w-full cursor-pointer visible bg-neutral-100">
        <video
          ref={videoRef}
          controls={false}
          autoPlay={false}
          muted
          loop
          style={{ width: '100%', height: '100%', objectFit: 'cover' }}
        >
          <source src={currentLibrary.getFileSrc(item.assetObjectHash)} />
        </video>
      </div>
      <div className="absolute top-0 left-0 w-full h-full px-4 py-2 bg-black/60 text-neutral-300 flex flex-col justify-between">
        <div className="overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">
          {item.materializedPath}{item.name}
        </div>
        <div className="overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">
          {formatDuration(item.startTime/1000)}
        </div>
      </div>
    </div>
  )
}

export default function Search() {
  const [searchPayload, setSearchPayload] = useState<SearchRequestPayload|null>(null)
  const queryRes = rspc.useQuery(['video.search.all', searchPayload!], {
    enabled: !!searchPayload
  })

  const searchInputRef = useRef<HTMLInputElement>(null)
  const [previewItem, setPreviewItem] = useState<SearchResultPayload | null>(null)

  const [keywordTyping, setKeywordTyping] = useState<string>('')

  const handleVideoClick = useCallback(
    (item: SearchResultPayload) => {
      setPreviewItem(item)
    },
    [setPreviewItem],
  )

  const handleSearch = useCallback(
    (text: string, recordType: string = "FrameCaption") => {
      if (text && recordType) {
        setSearchPayload({ text, recordType })
      }
    },
    [setSearchPayload],
  )

  const [focused, setFocused] = useState(false)

  return (
    <main className="flex h-full flex-col">
      <div className="flex h-12 items-center justify-start border-b border-neutral-100 px-4">
        <div className="flex w-1/4 select-none items-center">
          <div className="px-2 py-1">&lt;</div>
          <div className="px-2 py-1">&gt;</div>
          <div className="ml-2 text-sm">搜索</div>
        </div>
        <div className="w-80 relative">
          <form onSubmit={(e) => {
            e.preventDefault()
            handleSearch(keywordTyping)
            if (searchInputRef.current) {
              searchInputRef.current.blur()
              searchInputRef.current.value = ""
            }
          }} className="block">
            <input
              ref={searchInputRef}
              type="text"
              className="block w-full rounded-md bg-neutral-100 px-4 py-2 text-sm outline-none text-black"
              placeholder="搜索"
              onInput={(e) => setKeywordTyping(e.currentTarget.value)}
              onFocus={(e) => setFocused(true)}
              onBlur={(e) => setTimeout(() => setFocused(false), 200)}
            />
            {/* <button className="ml-4 px-6 bg-black text-white" type="submit">Search</button> */}
          </form>
          {focused && keywordTyping ? (
            <div className='absolute z-10 top-full w-full text-sm rounded-md p-1 bg-white shadow-md'>
              <div className="px-2 py-1 text-neutral-400">搜索类型</div>
              <div
                className="px-2 py-2 flex items-center justify-start text-neutral-800 hover:bg-neutral-100 rounded-sm"
                onClick={() => handleSearch(keywordTyping, "FrameCaption")}
              >
                <span className="text-neutral-400"><Icon.image className="w-4" /></span>
                <span className="mx-2">搜索视频内容</span>
                <strong>{keywordTyping}</strong>
              </div>
              <div
                className="px-2 py-2 flex items-center justify-start text-neutral-800 hover:bg-neutral-100 rounded-sm"
                onClick={() => handleSearch(keywordTyping, "Transcript")}
              >
                <span className="text-neutral-400"><Icon.microphone className="w-4" /></span>
                <span className="mx-2">搜索视频语音</span>
                <strong>{keywordTyping}</strong>
              </div>
            </div>
          ) : null}
        </div>
      </div>
      {searchPayload ? (
        <div className="px-8 py-2 flex items-center justify-start border-b border-neutral-100">
          <div className="border border-neutral-200 flex items-center text-xs rounded-lg overflow-hidden">
            <div
              className={classNames(
                "px-4 py-2",
                searchPayload.recordType === "FrameCaption" && "bg-neutral-100"
              )}
              onClick={() => handleSearch(searchPayload.text, "FrameCaption")}
            >视频内容</div>
            <div
              className={classNames(
                "px-4 py-2",
                searchPayload.recordType === "Transcript" && "bg-neutral-100"
              )}
              onClick={() => handleSearch(searchPayload.text, "Transcript")}
            >视频语音</div>
          </div>
          <div className="ml-4 text-sm text-neutral-600">{searchPayload.text}</div>
        </div>
      ) : null}
      <div className="p-8">
        {queryRes.isLoading ? (
          <div className="flex items-center justify-center px-2 py-8 text-sm text-neutral-400">正在搜索...</div>
        ) : (
          <div className="flex flex-wrap gap-4">
            {queryRes.data?.map((item: SearchResultPayload, index: number) => {
              return (
                <VideoItem
                  key={`${item.assetObjectId}-${index}`} item={item}
                  handleVideoClick={handleVideoClick}
                ></VideoItem>
              )
            })}
          </div>
        )}
      </div>
      {previewItem && (
        <div className="fixed left-0 top-0 flex h-full w-full items-center justify-center">
          <div
            className="absolute left-0 top-0 h-full w-full bg-black opacity-70"
            onClick={() => setPreviewItem(null)}
          ></div>
          <div className="relative h-[90%] w-[80%]">
            <VideoPreview item={previewItem}></VideoPreview>
          </div>
        </div>
      )}
    </main>
  )
}
