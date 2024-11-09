'use client'
import { useExplorerContext } from '@/Explorer/hooks'
import { uniqueId, type ExtractExplorerItem } from '@/Explorer/types'
import useDebouncedCallback from '@/hooks/useDebouncedCallback'
import type { SearchResultData } from '@/lib/bindings'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import SearchViewItem from './SearchViewItem'

export type ItemWithSize = {
  data: ExtractExplorerItem<'SearchResult'>
  width: number // width in px
  height: number // height in px
}

export default function SearchResults() {
  const ref = useRef<HTMLDivElement>(null)
  const gap = 10
  const padding = 0 // container 左右 padding
  const [containerWidth, setContainerWidth] = useState<number>(0)

  const explorer = useExplorerContext()

  const items = (explorer.items || []).filter(
    (item) => item.type === 'SearchResult',
  ) as ExtractExplorerItem<'SearchResult'>[]

  const debounceSetContainerWidth = useDebouncedCallback((containerWidth: number) => {
    setContainerWidth(containerWidth)
  }, 100)

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
          debounceSetContainerWidth(containerWidth)
        }
      }
    })
    resizeObserver.observe($el)
    return () => {
      resizeObserver.unobserve($el)
    }
  }, [debounceSetContainerWidth])

  const framesWidth = useCallback((metadata: SearchResultData['metadata'], singleWidth: number) => {
    // const startTime = Math.floor(metadata.startTime / 1e3)
    // const endTime = Math.floor(metadata.endTime / 1e3)
    // const duration = endTime - startTime
    // let repeat = 1
    // if (duration < 1) {
    //   //
    // } else if (duration >= 1 && duration < 6) {
    //   repeat = 2
    // } else if (duration >= 6) {
    //   repeat = 3
    // }
    // const width = repeat * singleWidth
    // return { width }
    // TODO 重新看下宽度如何计算
    return { width: singleWidth }
  }, [])

  const itemsWithSize = useMemo<ItemWithSize[]>(() => {
    if (!containerWidth) {
      return []
    }
    let itemsTotalWidth = 0
    let itemsWithSize: ItemWithSize[] = []
    let queue: ItemWithSize[] = []
    for (let data of items) {
      /* 单个 frame: 高度 100px, 宽度 150px */
      const { width } = framesWidth(data.metadata, 240)
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
      queue.push({ data, width, height })
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
      {itemsWithSize.map((item: ItemWithSize, index: number) => (
        <SearchViewItem key={uniqueId(item.data)} {...item}></SearchViewItem>
      ))}
    </div>
  )
}
