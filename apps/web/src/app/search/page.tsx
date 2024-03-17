'use client'
import type { SearchResultPayload } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { useCallback, useEffect, useRef, useState } from 'react'

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
      <source src={currentLibrary.getFileSrc(item.assetObjectHash)} type="video/mp4" />
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
    <div className="m-4 w-64">
      <div
        className="relative h-36 w-full cursor-pointer overflow-hidden rounded-md shadow-md"
        onClick={() => handleVideoClick(item)}
      >
        <video
          ref={videoRef}
          controls={false}
          autoPlay
          muted
          loop
          style={{ width: '100%', height: '100%', objectFit: 'cover' }}
        >
          <source src={currentLibrary.getFileSrc(item.assetObjectHash)} type="video/mp4" />
        </video>
      </div>
      <div
        className="mt-2 overflow-hidden overflow-ellipsis whitespace-nowrap
        px-2 text-center text-sm"
      >
        {item.name}
      </div>
      <div
        className="mb-2 overflow-hidden overflow-ellipsis whitespace-nowrap px-2
        text-center text-xs text-neutral-400"
      >
        {item.materializedPath}
      </div>
    </div>
  )
}

export default function Search() {
  const [searchKeyword, setSearchKeyword] = useState('')
  const queryRes = rspc.useQuery(['video.search.all', searchKeyword])
  // cosnt { data, isLoading, error } = queryRes;
  const searchInputRef = useRef<HTMLInputElement>(null)
  const [previewItem, setPreviewItem] = useState<SearchResultPayload | null>(null)

  const handleVideoClick = useCallback(
    (item: SearchResultPayload) => {
      setPreviewItem(item)
    },
    [setPreviewItem],
  )

  const handleSearch = useCallback(
    (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()
      const keyword = searchInputRef.current?.value
      if (!keyword) return
      setSearchKeyword(keyword)
    },
    [setSearchKeyword],
  )

  return (
    <main className="flex h-full flex-col">
      <div className="flex h-12 items-center justify-start border-b border-neutral-100 px-4">
        <div className="flex w-1/4 select-none items-center">
          <div className="px-2 py-1">&lt;</div>
          <div className="px-2 py-1">&gt;</div>
          <div className="ml-2 text-sm">搜索</div>
        </div>
        <div className="w-1/2">
          <form onSubmit={handleSearch} className="block">
            <input
              ref={searchInputRef}
              type="text"
              className="ml-auto mr-auto block w-80 rounded-md bg-neutral-100 px-4
                py-2 text-sm text-black"
              placeholder="搜索"
            />
            {/* <button className="ml-4 px-6 bg-black text-white" type="submit">Search</button> */}
          </form>
        </div>
      </div>
      <div className="p-6">
        {queryRes.isLoading ? (
          <div className="flex items-center justify-center px-2 py-8 text-sm text-neutral-400">正在搜索...</div>
        ) : (
          <div className="flex flex-wrap">
            {queryRes.data?.map((item: SearchResultPayload, index: number) => {
              return <VideoItem key={index} item={item} handleVideoClick={handleVideoClick}></VideoItem>
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
