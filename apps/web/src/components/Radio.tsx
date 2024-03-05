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
        'flex cursor-pointer items-center gap-2 rounded-[6px] border border-[#EBECEE] px-4 py-3 transition',
        !active ? 'hover:bg-[#EBECEE]' : 'bg-[#EBECEE]',
      )}
    >
      <div
        className={cn(
          'h-4 w-4 rounded-full border-[0.5px] border-[#C9C9C9] shadow-inner transition',
          active && 'border-[4px] border-[#017AFF] shadow-none',
        )}
      ></div>
      <div className="select-none text-[12px] font-medium leading-[14px] text-[#262626]">{label}</div>
    </div>
  )
}
