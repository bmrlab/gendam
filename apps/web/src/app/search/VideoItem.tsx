'use client'
import ViewItem from '@/Explorer/components/View/ViewItem'
import { useExplorerContext } from '@/Explorer/hooks'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import { useCurrentLibrary } from '@/lib/library'
import { formatDuration } from '@/lib/utils'
import classNames from 'classnames'
import Image from 'next/image'
import { useCallback, useMemo } from 'react'
import { type ItemWithSize } from './SearchResults'

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
    <ViewItem data={data} onClick={onSelect} onDoubleClick={() => quickview()}>
      <div
        className={classNames(
          'group relative overflow-hidden rounded-xl border-4',
          // 'transition-all duration-200 ease-in-out',
          highlight ? 'border-accent' : 'border-app-line/75',
        )}
        style={{ width: `${width}px`, height: `${height}px` }}
      >
        <div className="flex h-full items-stretch justify-between">
          {frames.map((frame, index) => (
            <div key={index} className="visible relative flex-1 bg-neutral-100">
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
