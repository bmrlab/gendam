import { cn } from '@/lib/utils'
import { HTMLAttributes } from 'react'

export type MuseRadioProps = {
  label: string
  active?: boolean
} & HTMLAttributes<HTMLDivElement>

export default function MuseRadio({ label, active, ...props }: MuseRadioProps) {
  return (
    <div
      {...props}
      className={cn(
        'flex cursor-pointer items-center gap-2 rounded border border-app-line px-4 py-3 transition',
        !active ? 'hover:bg-app-hover' : 'bg-app-hover',
      )}
    >
      <div
        className={cn(
          'h-4 w-4 rounded-full border border-app-line shadow-inner transition',
          active && 'border-4 border-accent shadow-none',
        )}
      ></div>
      <div className="select-none text-xs font-medium leading-4 text-ink">{label}</div>
    </div>
  )
}
