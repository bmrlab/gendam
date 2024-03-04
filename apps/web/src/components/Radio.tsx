import { cn } from '@/lib/utils'

export type MuseRadioProps = {
  active?: boolean
}

export default function MuseRadio({ active }: MuseRadioProps) {
  return (
    <div
      className={cn(
        'flex cursor-pointer items-center gap-2 rounded-[6px] border border-[#EBECEE] px-4 py-3',
        !active ? 'hover:bg-[#EBECEE]' : 'bg-[#EBECEE]',
      )}
    >
      <div
        className={cn(
          'h-4 w-4 rounded-full border-[0.5px] border-[#C9C9C9] shadow-inner',
          active && 'border-[4px] border-[#017AFF] shadow-none',
        )}
      ></div>
      <div className="text-[12px] font-medium leading-[14px] text-[#262626]">txt</div>
    </div>
  )
}
