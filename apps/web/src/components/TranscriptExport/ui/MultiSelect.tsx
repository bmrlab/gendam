import { cn } from '@/lib/utils'
import Icon from '@gendam/ui/icons'
import { MultiSelect } from '@gendam/ui/v2/multi-select'
import { FC } from 'react'

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
    <MultiSelect.Root {...props}>
      <MultiSelect.Trigger
        icon={(open) => <Icon.ArrowDown aria-hidden className={cn('h-4 w-4', open ? 'text-ink/50' : 'text-ink')} />}
        openClassName="ring-2 ring-accent"
        className="border-app-line w-full cursor-pointer px-2 py-2"
      >
        <MultiSelect.Value
          badge={(key, child) => (
            <div key={key} className="border-app-line block rounded border px-2.5 py-1">
              <p className="text-xs font-semibold uppercase leading-4">
                {showValue ? options.find((o) => o.label === (child as string))?.value : (child as string)}
              </p>
            </div>
          )}
          placeholder={placeholder}
          placeholderClassName="text-xs font-normal leading-4 text-ink"
          className="cursor-pointer gap-x-1 gap-y-1.5"
        />
      </MultiSelect.Trigger>
      <MultiSelect.Content
        contentClassName="shadow-md rounded-md bg-app-box border-app-line"
        className="cursor-pointer"
      >
        <MultiSelect.List className="py-2">
          {options.map((option) => (
            <MultiSelect.Item
              key={option.value}
              value={option.value}
              checkIcon={<Icon.Check className="h-4 w-4" />}
              className="aria-selected:bg-accent cursor-pointer rounded px-2 py-1 text-xs leading-4 aria-selected:text-white"
            >
              {option.label}
            </MultiSelect.Item>
          ))}
        </MultiSelect.List>
      </MultiSelect.Content>
    </MultiSelect.Root>
  )
}

export default MuseMultiSelect
