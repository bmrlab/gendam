import Icon from '@/components/Icon'
import { cn } from '@/lib/utils'
import { HTMLAttributes, useMemo } from 'react'

export enum TaskStatus {
  Failed,
  Cancelled,
  Done,
  Processing,
  None,
}

type Props = {
  status: TaskStatus
  name: string
} & HTMLAttributes<HTMLDivElement>

export function VideoTaskStatus({ status, name, className }: Props) {
  const {
    fgColor,
    bgColor,
    icon: IconComponent,
  } = useMemo(() => {
    switch (status) {
      case TaskStatus.Failed:
        return {
          fgColor: 'text-[#E61A1A]',
          bgColor: 'bg-[#FCEBEC]',
          icon: Icon.cross,
        }
      case TaskStatus.Cancelled:
        return {
          fgColor: 'text-[#000000]',
          bgColor: 'bg-[#F5F5F5]',
          icon: Icon.cross,
        }
      case TaskStatus.Done:
        return {
          fgColor: 'text-[#34C759]',
          bgColor: 'bg-[#EEF8E9]',
          icon: Icon.check,
        }
      case TaskStatus.Processing:
        return {
          fgColor: 'text-[#F27F0D]',
          bgColor: 'bg-[#FEF1EA]',
          icon: Icon.loading,
        }
      default:
        return {
          fgColor: 'text-[#000000]',
          bgColor: 'bg-[#F5F5F5]',
          icon: Icon.regenerate,
        }
    }
  }, [status])

  return (
    <div className={cn('flex items-center gap-0.5 rounded-[1000px] py-1 pl-2 pr-[10px]', fgColor, bgColor, className)}>
      <IconComponent className={cn(status === TaskStatus.Processing && 'animate-spin')} />
      <p className="text-xs leading-4">{name}</p>
    </div>
  )
}
