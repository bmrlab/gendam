import { cn } from '@/lib/utils'
import { HTMLAttributes } from 'react'

export function RoundedBadge({ children, className, ...props }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      {...props}
      className={cn(
        'cursor-pointer rounded-[1000px] bg-black/60 px-2 py-[5px] text-[12px] font-normal leading-4 text-white transition hover:bg-black/80 hover:shadow-sm',
        className,
      )}
    >
      {children}
    </div>
  )
}
