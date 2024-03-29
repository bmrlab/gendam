import { Filter } from '@/lib/bindings'
import { Label } from '@muse/ui/v1/label'
import { RadioGroup, RadioGroupItem } from '@muse/ui/v1/radio-group'
import { useMemo } from 'react'
import { useBoundStore } from '../_store'

export default function FilterWidget() {
  const filter = useBoundStore.use.taskFilter()
  const setFilter = useBoundStore.use.setTaskFilter()

  const filterOptions = useMemo<
    {
      value: Filter
      label: string
    }[]
  >(
    () => [
      {
        value: 'all',
        label: '全部',
      },
      {
        value: 'excludeCompleted',
        label: '未完成',
      },
    ],
    [],
  )

  return (
    <div>
      <RadioGroup
        defaultValue={filter as string}
        className="flex px-8 py-2"
        value={filter as string}
        onValueChange={(filter) => setFilter(filter as Filter)}
      >
        {filterOptions.map((option) => {
          let key = option.value as string
          return (
            <div key={key} className="flex items-center space-x-2">
              <RadioGroupItem value={key} id={key} />
              <Label htmlFor={key}>{option.label}</Label>
            </div>
          )
        })}
      </RadioGroup>
    </div>
  )
}
