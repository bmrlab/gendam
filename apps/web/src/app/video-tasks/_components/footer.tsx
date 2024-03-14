'use client'
import { useBoundStore } from '@/store'

export default function TaskFooter({ total }: { total: number }) {
  const selected = useBoundStore.use.videoSelected()

  return (
    <div className="h-[28px] w-full border-t border-[#DDDDDE] bg-[#F6F7F9] py-[7px] text-center text-[11px] font-normal leading-[14px] text-[#676C77]">
      <p>
        已选中 {selected.length ?? 0}/{total} 个视频
      </p>
    </div>
  )
}
