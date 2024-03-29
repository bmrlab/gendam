'use client'
import Viewport from '@/components/Viewport'
import { useBoundStore } from '../_store'

export default function TaskFooter({ total }: { total: number }) {
  const selected = useBoundStore.use.videoSelected()

  return (
    <Viewport.StatusBar className="justify-center">
      <div className="text-xs text-neutral-500">
        已选中 {selected.length ?? 0}/{total} 个视频
      </div>
    </Viewport.StatusBar>
  )
}
