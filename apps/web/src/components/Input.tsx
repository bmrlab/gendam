import { SvgIconProps } from '@/components/Icon'
import { Input, InputProps } from '@/components/ui/input'
import { cn, twx } from '@/lib/utils'
import { ComponentType } from 'react'

export type MuseInputProps = {
  icon?: ComponentType<SvgIconProps>
} & InputProps

export default function MuseInput({ icon: Icon, className, ...props }: MuseInputProps) {
  return (
    <div className="relative">
      {Icon && (
        <div className="absolute left-2 top-1/2 size-[14px] translate-y-[-50%] text-[#676C77]">
          <Icon />
        </div>
      )}
      <_MuseInput {...props} className={cn('h-full px-2 py-[7px]', Icon && 'pl-[28px]', className)} />
    </div>
  )
}

const _MuseInput = twx(Input)`max-w-[320px] bg-black/5 border-[0.5px] border-black/5 caret-[#017AFF]
placeholder:text-[#676C77] placeholder:font-normal placeholder:text-[13px] placeholder:leading-[16px]
focus:outline-none focus-visible:ring-2 focus-visible:ring-[#017AFF]
`
