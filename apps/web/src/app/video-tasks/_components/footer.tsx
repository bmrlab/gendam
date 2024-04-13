'use client'
import Viewport from '@/components/Viewport'
import { useBoundStore } from '../_store'

export default function TaskFooter({ total }: { total: number }) {
  const selected = useBoundStore.use.videoSelected()

  return (
    <Viewport.StatusBar className="justify-center">
      <div className="text-xs text-neutral-500">
        {selected.length ?? 0} of {total} jobs selected
      </div>
    </Viewport.StatusBar>
  )
}
