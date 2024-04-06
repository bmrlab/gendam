import Icon from '@/components/Icon'
import { useCurrentLibrary } from '@/lib/library'
import { useEffect, useRef } from 'react'
import { useQuickViewStore, type QuickViewItem } from './store'

const Player = ({ data }: { data: QuickViewItem }) => {
  const currentLibrary = useCurrentLibrary()

  const videoRef = useRef<HTMLVideoElement | null>(null)
  useEffect(() => {
    const $video = videoRef?.current
    if (!$video) {
      return
    }
    const startTime = Math.max(0, (data.video?.currentTime || 0) - 0.5)
    // const endTime = startTime + 2
    const videoSrc = currentLibrary.getFileSrc(data.assetObject.hash)
    // 重新赋值才能在 src 变化了以后重新加载视频
    if ($video.src != videoSrc) {
      $video.src = videoSrc
      $video.currentTime = startTime
      // $video.ontimeupdate = () => {
      //   if ($video.currentTime >= endTime) {
      //     $video.pause()
      //     $video.ontimeupdate = null
      //   }
      // }
    }
  }, [currentLibrary, data])

  return (
    <div className="flex h-full w-full items-center justify-center overflow-hidden">
      <video
        ref={videoRef}
        controls
        controlsList="nodownload"
        autoPlay
        muted
        className="h-auto max-h-full w-auto max-w-full overflow-hidden rounded-md"
      >
        {/* <source src={currentLibrary.getFileSrc(assetObject.hash)} /> */}
      </video>
    </div>
  )
}

export default function QuickView() {
  const quickViewStore = useQuickViewStore()

  // quickViewStore.show === true 的时候 quickViewStore.data 不会为空，这里只是为了下面 tsc 检查通过
  return quickViewStore.show && quickViewStore.data ? (
    <div className="fixed left-0 top-0 h-full w-full bg-black/50 px-20 py-10" onClick={() => quickViewStore.close()}>
      <div
        className="relative h-full w-full rounded-lg bg-black/50 px-8 pb-8 pt-20 shadow backdrop-blur-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="absolute left-0 top-6 w-full overflow-hidden px-12 text-center font-medium text-white/90">
          <div className="truncate">{quickViewStore.data.name}</div>
        </div>
        <Player data={quickViewStore.data} />
        <div
          className="absolute right-0 top-0 flex h-12 w-12 items-center justify-center p-2 hover:opacity-70"
          onClick={() => quickViewStore.close()}
        >
          <Icon.cross className="h-6 w-6 text-white/50" />
        </div>
      </div>
    </div>
  ) : (
    <></>
  )
}
