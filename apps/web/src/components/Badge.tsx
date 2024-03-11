import Icon from '@/components/Icon'
import { cn } from '@/lib/utils'
import { HTMLAttributes, useMemo } from 'react'

export enum MuseStatus {
  Processing,
  Done,
  Failed,
}

export type MuseBadgeProps = {
  status: MuseStatus
  name: string
} & HTMLAttributes<HTMLDivElement>

export function MuseTaskBadge({ status, name, className }: MuseBadgeProps) {
  const {
    fgColor,
    bgColor,
    icon: IconComponent,
  } = useMemo(() => {
    switch (status) {
      case MuseStatus.Done:
        return {
          fgColor: 'text-[#34C759]',
          bgColor: 'bg-[#EEF8E9]',
          icon: Icon.check,
        }
      case MuseStatus.Failed:
        return {
          fgColor: 'text-[#E61A1A]',
          bgColor: 'bg-[#FCEBEC]',
          icon: Icon.error,
        }
      default:
        return {
          fgColor: 'text-[#F27F0D]',
          bgColor: 'bg-[#FEF1EA]',
          icon: Icon.loading,
        }
    }
  }, [status])

  return (
    <div className={cn('flex items-center gap-0.5 rounded-[1000px] py-1 pl-2 pr-[10px]', fgColor, bgColor, className)}>
      <IconComponent className={cn(status === MuseStatus.Processing && 'animate-spin')} />
      <p className="text-[12px] leading-4">{name}</p>
    </div>
  )
}

export function MuseBadge({ children, className, ...props }: HTMLAttributes<HTMLDivElement>) {
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
