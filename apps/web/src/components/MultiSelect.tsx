import {
  MultiSelect,
  MultiSelectContent,
  MultiSelectItem,
  MultiSelectList,
  MultiSelectProps,
  MultiSelectTrigger,
  MultiSelectValue,
} from '@/components/ui/multi-select'
import { cn } from '@/lib/utils'
import { FC } from 'react'
import Icon from './Icon'

export type MuseMultiSelectProps = MultiSelectProps & {
  options: { label: string; value: string }[]
  placeholder?: string
  // 选中之后，显示 label 还是 value ？
  // 默认是 label
  showValue?: boolean
}

// eslint-disable-next-line react/display-name
const MuseMultiSelect: FC<MuseMultiSelectProps> = ({ showValue, options, placeholder, ...props }) => {
  return (
    <MultiSelect {...props}>
      <MultiSelectTrigger
        icon={(open) => (
          <Icon.arrowDown aria-hidden className={cn('h-4 w-4', open ? 'text-[#262626]' : 'text-[#95989F]')} />
        )}
        openClassName="ring-2 ring-[#017AFF]"
        className="w-full cursor-pointer border-[#DDDDDE] px-2 py-[6px]"
      >
        <MultiSelectValue
          badge={(key, child) => (
            <div key={key} className="block rounded border border-[#DDDDDE] px-2.5 py-1">
              <p className="text-[12px] font-semibold uppercase leading-[12px]">
                {showValue ? options.find((o) => o.label === (child as string))?.value : (child as string)}
              </p>
            </div>
          )}
          placeholder={placeholder}
          placeholderClassName="text-[13px] font-normal leading-[18px] text-[#676C77]"
          className="cursor-pointer gap-x-1 gap-y-1.5"
        />
      </MultiSelectTrigger>
      <MultiSelectContent
        contentClassName="shadow-md rounded-[6px] border-[#DDDDDE]"
        className="cursor-pointer bg-[#F4F5F5]"
      >
        <MultiSelectList className="py-2">
          {options.map((option) => (
            <MultiSelectItem
              key={option.value}
              value={option.value}
              checkIcon={<Icon.checked className="h-4 w-4" />}
              className="cursor-pointer rounded-[6px] px-2.5 py-[2.5px] text-[13px] font-medium leading-[19.5px]  aria-selected:bg-[#017AFF] aria-selected:text-white"
            >
              {option.label}
            </MultiSelectItem>
          ))}
        </MultiSelectList>
      </MultiSelectContent>
    </MultiSelect>
  )
}

export default MuseMultiSelect
