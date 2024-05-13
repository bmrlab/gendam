import {
  MultiSelect,
  MultiSelectContent,
  MultiSelectItem,
  MultiSelectList,
  MultiSelectProps,
  MultiSelectTrigger,
  MultiSelectValue,
} from '@gendam/ui/v1/multi-select'
import { cn } from '@/lib/utils'
import { FC } from 'react'
import Icon from '@gendam/ui/icons'

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
          <Icon.ArrowDown aria-hidden className={cn('h-4 w-4', open ? 'text-ink/50' : 'text-ink')} />
        )}
        openClassName="ring-2 ring-accent"
        className="w-full cursor-pointer border-app-line px-2 py-2"
      >
        <MultiSelectValue
          badge={(key, child) => (
            <div key={key} className="block rounded border border-app-line px-2.5 py-1">
              <p className="text-xs font-semibold uppercase leading-4">
                {showValue ? options.find((o) => o.label === (child as string))?.value : (child as string)}
              </p>
            </div>
          )}
          placeholder={placeholder}
          placeholderClassName="text-xs font-normal leading-4 text-ink"
          className="cursor-pointer gap-x-1 gap-y-1.5"
        />
      </MultiSelectTrigger>
      <MultiSelectContent
        contentClassName="shadow-md rounded-md bg-app-box border-app-line"
        className="cursor-pointer"
      >
        <MultiSelectList className="py-2">
          {options.map((option) => (
            <MultiSelectItem
              key={option.value}
              value={option.value}
              checkIcon={<Icon.Check className="h-4 w-4" />}
              className="cursor-pointer rounded px-2 py-1 text-xs leading-4 aria-selected:bg-accent aria-selected:text-white"
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
