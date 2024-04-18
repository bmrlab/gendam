'use client'
// import Icon from '@muse/ui/icons'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import classNames from 'classnames'
import type { SearchResultPayload } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { formatDuration } from '@/lib/utils'
import Image from 'next/image'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'

type ItemsWithSize = {
  data: SearchResultPayload
  width: number     // width in px
  height: number    // height in px
  frames: number[]  // frame timestamps
}

const VideoItem: React.FC<{ item: ItemsWithSize }> = ({ item }) => {
  const quickViewStore = useQuickViewStore()
  const currentLibrary = useCurrentLibrary()
  const videoRef = useRef<HTMLVideoElement>(null)

  useEffect(() => {
    const video = videoRef.current
    if (!video) return
    let startTime = Math.max(0, item.data.startTime / 1e3 - 0.5)
    let endTime = Math.max(startTime, item.data.endTime / 1e3 + 1.5)
    video.currentTime = startTime
    video.ontimeupdate = () => {
      if (video.currentTime >= endTime) {
        // video.pause();
        // video.ontimeupdate = null;
        video.currentTime = startTime
      }
    }
  }, [item.data])

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

  return (
    <div
      className={classNames("group relative overflow-hidden rounded-xl border-4 border-app-line/75")}
      // style={{ minWidth: `${width}rem`, height: '10rem', flex: frames.length }}
      style={{ width: `${item.width}px`, height: `${item.height}px` }}
      onClick={() => handleVideoClick(item.data)}
    >
      <div className="flex items-stretch justify-between h-full">
        {item.frames.map((frame, index) => (
          <div
            key={index}
            className="visible relative flex-1 cursor-pointer bg-neutral-100"
          >
            <Image
              src={currentLibrary.getThumbnailSrc(item.data.assetObjectHash, frame)}
              alt={item.data.name}
              fill={true}
              className="object-cover"
              priority
            ></Image>
          </div>
        ))}
      </div>
      <div
        className={classNames(
          "absolute left-0 top-0 flex h-full w-full flex-col justify-between bg-black/60 px-4 py-2 text-neutral-300",
          "invisible group-hover:visible"
        )}
      >
        <div className="truncate text-xs">
          {item.data.materializedPath}
          {item.data.name}
        </div>
        <div className='flex items-center justify-between text-xs'>
          <div>{formatDuration(item.data.startTime / 1000)}</div>
          <div>→</div>
          <div>{formatDuration(item.data.endTime / 1000 + 1)}</div>
        </div>
      </div>
    </div>
  )
}

export default function SearchResults({ items, groupFrames }: {
  items: SearchResultPayload[]
  groupFrames: boolean
}) {
  const ref = useRef<HTMLDivElement>(null)
  const gap = 10
  const padding = 0  // container 左右 padding
  const [containerWidth, setContainerWidth] = useState<number>(0)

  useEffect(() => {
    const $el = ref.current;
    if (!$el) {
      return
    }
    // ref.current 必须在 useEffect 里面用, 不然还没有 mount, 它还是 undefined
    const containerWidth = Math.max(0, ($el.clientWidth || 0) - padding * 2)
    setContainerWidth(containerWidth)
    const resizeObserver = new ResizeObserver(entries => {
      for (let entry of entries) {
        if (entry.target === $el) {
          const containerWidth = Math.max(0, ($el.clientWidth || 0) - padding * 2)
          setContainerWidth(containerWidth)
        }
      }
    });
    resizeObserver.observe($el);
    return () => {
      resizeObserver.unobserve($el);
    };
  }, [])

  const framesWidth = useCallback((item: SearchResultPayload, singleWidth: number) => {
    const startTime = Math.floor(item.startTime / 1e3)
    const endTime = Math.floor(item.endTime / 1e3)
    const duration = endTime - startTime
    let repeat = 1;
    let frames = [startTime];
    if (!groupFrames || duration < 1) {
      //
    } else if (duration >= 1 && duration < 6) {
      repeat = 2
      frames = [startTime, endTime]
    } else if (duration >= 6) {
      repeat = 3
      frames = [startTime, Math.floor((startTime + endTime) / 2), endTime]
    }
    const width = repeat * singleWidth
    return { frames, width }
  }, [groupFrames])

  const itemsWithSize = useMemo<ItemsWithSize[]>(() => {
    if (!containerWidth) {
      return []
    }
    let itemsTotalWidth = 0
    let itemsWithSize: ItemsWithSize[] = []
    let queue: ItemsWithSize[] = []
    for (let data of items) {
      /* 单个 frame: 高度 100px, 宽度 150px */
      const { frames, width } = framesWidth(data, 240)
      const height = 160
      const maxTotalWidth = containerWidth - (gap * (queue.length - 1))
      if (itemsTotalWidth + width > maxTotalWidth) {
        const scale = maxTotalWidth / itemsTotalWidth
        queue.forEach((item) => {
          item.width *= scale
          item.height *= scale
        })
        itemsWithSize = itemsWithSize.concat(queue)
        itemsTotalWidth = 0
        queue.length = 0
      }
      itemsTotalWidth += width
      queue.push({ data, width, height, frames })
    }
    itemsWithSize = [ ...itemsWithSize, ...queue]
    return itemsWithSize
  }, [containerWidth, framesWidth, items])

  console.log(containerWidth, itemsWithSize)

  return (
    <div
      ref={ref}
      className="min-h-full pb-8 flex flex-wrap content-start"
      style={{ columnGap: `${gap}px`, rowGap: '30px' }}
    >
      {itemsWithSize.map((item: ItemsWithSize, index: number) => (
        <VideoItem key={`${item.data.assetObjectId}-${index}`} item={item}></VideoItem>
      ))}
    </div>
  )
}
