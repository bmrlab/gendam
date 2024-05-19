'use client'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import { useCurrentLibrary } from '@/lib/library'
import { formatDuration } from '@/lib/utils'
import classNames from 'classnames'
import Image from 'next/image'
import { useCallback, useEffect, useMemo, useRef } from 'react'
import { type ItemWithSize } from './SearchResults'
import { uniqueId } from '@/Explorer/types'

const VideoItem: React.FC<ItemWithSize> = ({ data, width, height, frames }) => {
  const explorer = useExplorerContext()
  const quickViewStore = useQuickViewStore()
  const currentLibrary = useCurrentLibrary()

  const { filePath, metadata } = data

  const highlight = useMemo(() => {
    return explorer.isItemSelected(data)
  }, [data, explorer])

  const quickview = useCallback(() => {
    quickViewStore.open({
      name: filePath.name,
      assetObject: filePath.assetObject!,
      video: {
        currentTime: metadata.startTime / 1e3,
      },
    })
  }, [quickViewStore, filePath, metadata])

  const onSelect = useCallback(
    (e: React.MouseEvent) => {
      // ExplorerLayout 上面有一个 onClick={resetSelectedItems} 会清空选中的项目, 这里一定要 stop 一下
      e.stopPropagation()
      // 按住 cmd 键多选
      if (e.metaKey) {
        if (explorer.isItemSelected(data)) {
          explorer.removeSelectedItem(data)
        } else {
          explorer.addSelectedItem(data)
        }
      } else {
        explorer.resetSelectedItems([data])
      }
      // explorerStore.reset()
    },
    [explorer, data],
  )

  return (
    <ViewItem data={data}>
      <div
        data-selecto-item={uniqueId(data)}
        data-component-hint="ViewItem(SearchResultItem)"
        onDoubleClick={() => quickview()}
        onClick={onSelect}
        className={classNames(
          'group relative overflow-hidden rounded-xl border-4',
          // 'transition-all duration-200 ease-in-out',
          highlight ? 'border-accent' : 'border-app-line/75',
        )}
        // style={{ minWidth: `${width}rem`, height: '10rem', flex: frames.length }}
        style={{ width: `${width}px`, height: `${height}px` }}
      >
        <div className="flex h-full items-stretch justify-between">
          {frames.map((frame, index) => (
            <div key={index} className="visible relative flex-1 cursor-pointer bg-neutral-100">
              <Image
                src={currentLibrary.getThumbnailSrc(filePath.assetObject?.hash!, frame)}
                alt={filePath.name}
                fill={true}
                className="object-cover"
                priority
              ></Image>
            </div>
          ))}
        </div>
        <div
          className={classNames(
            'absolute left-0 top-0 flex h-full w-full flex-col justify-between bg-black/60 px-4 py-2 text-neutral-300',
            'invisible group-hover:visible',
          )}
        >
          <div className="truncate text-xs">
            {filePath.materializedPath}
            {filePath.name}
          </div>
          <div className="flex items-center justify-between text-xs">
            <div>{formatDuration(metadata.startTime / 1000)}</div>
            <div>→</div>
            <div>{formatDuration(metadata.endTime / 1000 + 1)}</div>
          </div>
        </div>
      </div>
    </ViewItem>
  )
}

export default VideoItem
