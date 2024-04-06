// import { ExplorerItem } from "@/Explorer/types";
// import { useState } from "react";
import { ExplorerItem } from '@/Explorer/types'
import Icon from '@/components/Icon'
import { useCurrentLibrary } from '@/lib/library'
import { createRef, useEffect } from 'react'
import { useQuickViewStore } from './store'

const Player = ({ data }: { data: ExplorerItem }) => {
  const currentLibrary = useCurrentLibrary()

  const videoRef = createRef<HTMLVideoElement>()
  useEffect(() => {
    if (!videoRef.current || !data.assetObject?.hash) {
      return
    }
    const videoSrc = currentLibrary.getFileSrc(data.assetObject.hash)
    // 重新赋值才能在 src 变化了以后重新加载视频
    if (videoRef.current.src != videoSrc) {
      videoRef.current.src = videoSrc
    }
  }, [currentLibrary, data, videoRef])

  if (!data.assetObject || !data.assetObject.mediaData) {
    return
  }
  const {
    assetObject,
    assetObject: { mediaData },
  } = data

  return (
    <div className="h-full w-full overflow-hidden flex items-center justify-center">
      <video
        ref={videoRef}
        controls
        controlsList="nodownload"
        autoPlay
        muted
        className="w-auto h-auto max-h-full max-w-full rounded-md overflow-hidden"
      >
        {/* <source src={currentLibrary.getFileSrc(assetObject.hash)} /> */}
      </video>
    </div>
  )
}

export default function QuickView() {
  const quickViewStore = useQuickViewStore()

  return quickViewStore.show ? (
    <div className="fixed left-0 top-0 h-full w-full bg-black/50 px-20 py-10" onClick={() => quickViewStore.close()}>
      <div
        className="relative h-full w-full rounded-lg bg-black/50 px-8 pb-8 pt-20 shadow backdrop-blur-xl"
        onClick={(e) => e.stopPropagation()}
      >
        <div className="absolute left-0 top-6 w-full px-12 overflow-hidden text-center font-medium text-white/90">
          <div className='truncate'>{quickViewStore.data?.name}</div>
        </div>
        <Player data={quickViewStore.data!} />
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
