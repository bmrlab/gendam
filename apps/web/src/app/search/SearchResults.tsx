'use client'
import type { SearchResultPayload } from '@/lib/bindings'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import VideoItem from './VideoItem'

export type ItemsWithSize = {
  filePath: SearchResultPayload['filePath']
  metadata: SearchResultPayload['metadata']
  width: number // width in px
  height: number // height in px
  frames: number[] // frame timestamps
}

export default function SearchResults({ items, groupFrames }: { items: SearchResultPayload[]; groupFrames: boolean }) {
  const ref = useRef<HTMLDivElement>(null)
  const gap = 10
  const padding = 0 // container 左右 padding
  const [containerWidth, setContainerWidth] = useState<number>(0)

  useEffect(() => {
    const $el = ref.current
    if (!$el) {
      return
    }
    // ref.current 必须在 useEffect 里面用, 不然还没有 mount, 它还是 undefined
    const containerWidth = Math.max(0, ($el.clientWidth || 0) - padding * 2)
    setContainerWidth(containerWidth)
    const resizeObserver = new ResizeObserver((entries) => {
      for (let entry of entries) {
        if (entry.target === $el) {
          const containerWidth = Math.max(0, ($el.clientWidth || 0) - padding * 2)
          setContainerWidth(containerWidth)
        }
      }
    })
    resizeObserver.observe($el)
    return () => {
      resizeObserver.unobserve($el)
    }
  }, [])

  const framesWidth = useCallback(
    (metadata: SearchResultPayload['metadata'], singleWidth: number) => {
      const startTime = Math.floor(metadata.startTime / 1e3)
      const endTime = Math.floor(metadata.endTime / 1e3)
      const duration = endTime - startTime
      let repeat = 1
      let frames = [startTime]
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
    },
    [groupFrames],
  )

  const itemsWithSize = useMemo<ItemsWithSize[]>(() => {
    if (!containerWidth) {
      return []
    }
    let itemsTotalWidth = 0
    let itemsWithSize: ItemsWithSize[] = []
    let queue: ItemsWithSize[] = []
    for (let { filePath, metadata } of items) {
      /* 单个 frame: 高度 100px, 宽度 150px */
      const { frames, width } = framesWidth(metadata, 240)
      const height = 160
      const maxTotalWidth = containerWidth - gap * (queue.length - 1)
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
      queue.push({ filePath, metadata, width, height, frames })
    }
    itemsWithSize = [...itemsWithSize, ...queue]
    return itemsWithSize
  }, [containerWidth, framesWidth, items])

  return (
    <div
      ref={ref}
      className="flex min-h-full flex-wrap content-start pb-8"
      style={{ columnGap: `${gap}px`, rowGap: '30px' }}
    >
      {itemsWithSize.map((item: ItemsWithSize, index: number) => (
        <VideoItem key={`${item.filePath.id}-${index}`} item={item}></VideoItem>
      ))}
    </div>
  )
}
