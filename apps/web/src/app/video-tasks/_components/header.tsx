import { cn } from '@/lib/utils'
import { HTMLAttributes } from 'react'

export default function VideoTaskHeader({ className }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div className={cn('flex justify-between border-b border-neutral-100 px-4', className)}>
      <div className="flex select-none items-center">
        <div className="px-2 py-1">&lt;</div>
        <div className="px-2 py-1">&gt;</div>
        <div className="ml-2 text-sm">任务列表</div>
      </div>
    </div>
  )
}
