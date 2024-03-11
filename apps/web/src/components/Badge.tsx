import Icon from '@/components/Icon'
import { cn } from '@/lib/utils'
import { useMemo } from 'react'

export enum MuseStatus {
  Processing,
  Done,
  Failed,
}

export type MuseBadgeProps = {
  status: MuseStatus
  name: string
}

export default function MuseBadge({ status, name }: MuseBadgeProps) {
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
    <div className={cn('flex items-center gap-0.5 rounded-[1000px] py-1 pl-2 pr-[10px]', fgColor, bgColor)}>
      <IconComponent className={cn(status === MuseStatus.Processing && 'animate-spin')} />
      <p>{name}</p>
    </div>
  )
}
