'use client'
import Icon from '@gendam/ui/icons'
import { CommandPrimitive } from '@gendam/ui/v2/command'
import classNames from 'classnames'
import { type SearchPayload } from './context'
// import classNames from 'classnames'
import { forwardRef, useCallback, useImperativeHandle, useRef, useState } from 'react'

type FormData = {
  text: string
  recordType: 'Frame' | 'Transcript'
}

export type SearchFormRef = {
  getValue: () => FormData | null
  setValue: (value: FormData | null) => void
}

const SearchFormWithRef = forwardRef<
  SearchFormRef,
  {
    onSubmit: () => void
  }
>(function SearchForm({ onSubmit }, ref) {
  const [text, setText] = useState('')
  const [recordType, setRecordType] = useState<'Frame' | 'Transcript' | null>(null)
  // const [value, setValue] = useState<T | null>(null)

  useImperativeHandle<SearchFormRef, SearchFormRef>(ref, () => ({
    getValue: () => (text && recordType ? { api: 'search.all', text, recordType } : null),
    setValue: (value) => {
      if (value) {
        setText(value.text)
        setRecordType(value.recordType)
      } else {
        setText('')
        setRecordType(null)
      }
    },
  }))

  const searchInputRef = useRef<HTMLInputElement>(null)
  const [typing, setTyping] = useState(false)

  const onSelectCommandItem = useCallback(
    (recordType: 'Frame' | 'Transcript') => {
      setRecordType(recordType)
      setTyping(false)
      searchInputRef.current?.blur()
      setTimeout(() => onSubmit(), 0)
    },
    [onSubmit],
  )

  const onClearValue = useCallback(() => {
    setText('')
    setRecordType(null)
    setTimeout(() => onSubmit(), 0)
  }, [onSubmit])

  return (
    <div className="relative mx-auto block w-96 max-w-full">
      <CommandPrimitive shouldFilter={false}>
        <div cmdk-input-wrapper="" className="relative">
          <CommandPrimitive.Input
            ref={searchInputRef}
            className="border-app-line bg-app-overlay text-ink block w-full rounded-md border px-4 py-[0.3rem] pl-7 pr-7 text-sm outline-none"
            placeholder="Search"
            value={text}
            onValueChange={(text) => setText(text)}
            onFocus={() => setTyping(true)}
            onBlur={() => setTimeout(() => setTyping(false), 200)}
            autoFocus={false}
          />
          <Icon.MagnifyingGlass className="text-ink/50 absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 transform" />
          <Icon.Close
            className={classNames(
              'text-ink/30 absolute right-2 top-1/2 h-4 w-4 -translate-y-1/2 transform',
              !text ? 'hidden' : null,
            )}
            onClick={() => onClearValue()}
          />
        </div>
        {typing && (
          <div className="border-app-line bg-app-box absolute top-full z-10 w-full rounded-md border p-1 text-sm shadow-md">
            <div className="text-ink/50 px-2 py-1">Search types</div>
            <CommandPrimitive.List>
              <CommandPrimitive.Item
                className="text-ink hover:bg-app-hover data-[selected]:bg-app-hover flex items-center justify-start overflow-hidden rounded-md px-2 py-2"
                onSelect={(e) => onSelectCommandItem('Frame')}
              >
                <div className="text-ink/50">
                  <Icon.Image className="w-4" />
                </div>
                <div className="mx-2 flex-1 break-all">
                  <span>Visual search for </span>
                  <strong>{text}</strong>
                </div>
              </CommandPrimitive.Item>
              <CommandPrimitive.Item
                className="text-ink hover:bg-app-hover data-[selected]:bg-app-hover flex items-center justify-start overflow-hidden rounded-md px-2 py-2"
                onSelect={(e) => onSelectCommandItem('Transcript')}
              >
                <div className="text-ink/50">
                  <Icon.Mic className="w-4" />
                </div>
                <div className="mx-2 flex-1 break-all">
                  <span>Transcript search for </span>
                  <strong>{text}</strong>
                </div>
              </CommandPrimitive.Item>
            </CommandPrimitive.List>
          </div>
        )}
      </CommandPrimitive>
    </div>
  )
})

export default SearchFormWithRef
