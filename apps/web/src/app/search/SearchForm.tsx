'use client'
import Icon from '@gendam/ui/icons'
import { SearchRequestPayload } from '@/lib/bindings'
import { CommandPrimitive } from '@gendam/ui/v2/command'
// import classNames from 'classnames'
import { useCallback, useEffect, useRef, useState } from 'react'

export default function SearchForm({
  initialSearchPayload,
  onSubmit,
}: {
  initialSearchPayload: SearchRequestPayload | null
  onSubmit: (text: string, recordType: string) => void
}) {
  const searchInputRef = useRef<HTMLInputElement>(null)
  const [keyword, setKeyword] = useState(initialSearchPayload?.text || '')
  const [typing, setTyping] = useState(false)

  const onSelectCommandItem = useCallback((recordType: string) => {
    onSubmit(keyword, recordType)
    setTyping(false)
    searchInputRef.current?.blur()
  }, [onSubmit, keyword, setTyping])

  return (
    <div className="relative w-96 max-w-full block mx-auto">
      <CommandPrimitive shouldFilter={false}>
        <div cmdk-input-wrapper="" className="relative">
          <CommandPrimitive.Input
            ref={searchInputRef}
            className="border-app-line bg-app-overlay text-ink block w-full rounded-md border px-4 py-[0.3rem] pl-7 text-sm outline-none"
            placeholder="Search"
            value={keyword}
            onValueChange={setKeyword}
            onFocus={() => setTyping(true)}
            onBlur={() => setTimeout(() => setTyping(false), 200)}
            autoFocus={false}
          />
          <Icon.MagnifyingGlass className="text-ink/50 absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 transform" />
        </div>
        {typing && (
          <div className="border-app-line bg-app-box absolute top-full z-10 w-full rounded-md border p-1 text-sm shadow-md">
            <div className="text-ink/50 px-2 py-1">Search types</div>
            <CommandPrimitive.List>
              <CommandPrimitive.Item
                className="text-ink hover:bg-app-hover data-[selected]:bg-app-hover flex items-center justify-start rounded-md px-2 py-2 overflow-hidden"
                onSelect={(e) => onSelectCommandItem('Frame')}
              >
                <div className="text-ink/50">
                  <Icon.Image className="w-4" />
                </div>
                <div className='mx-2 flex-1 break-all'>
                  <span>Visual search for </span>
                  <strong>{keyword}</strong>
                </div>
              </CommandPrimitive.Item>
              <CommandPrimitive.Item
                className="text-ink hover:bg-app-hover data-[selected]:bg-app-hover flex items-center justify-start rounded-md px-2 py-2 overflow-hidden"
                onSelect={(e) => onSelectCommandItem('Transcript')}
              >
                <div className="text-ink/50">
                  <Icon.Mic className="w-4" />
                </div>
                <div className='mx-2 flex-1 break-all'>
                  <span>Transcript search for </span>
                  <strong>{keyword}</strong>
                </div>
              </CommandPrimitive.Item>
            </CommandPrimitive.List>
          </div>
        )}
      </CommandPrimitive>
    </div>
  )
}

/*
<form
  onSubmit={(e) => {
    e.preventDefault()
    handleSearch(keywordTyping)
    if (searchInputRef.current) {
      searchInputRef.current.blur()
      searchInputRef.current.value = ''
    }
  }}
  className="relative block"
>
  <input
    ref={searchInputRef}
    type="text"
    className="block w-full rounded-md border border-app-line bg-app-overlay px-4 py-[0.3rem] pl-7 text-sm text-ink outline-none"
    placeholder="搜索"
    onInput={(e) => setKeywordTyping(e.currentTarget.value)}
    onFocus={(e) => setFocused(true)}
    onBlur={(e) => setTimeout(() => setFocused(false), 200)}
  />
  <Icon.search className="absolute left-2 top-1/2 h-4 w-4 -translate-y-1/2 transform text-ink/50" />
</form>

<div className="absolute top-full z-10 w-full rounded-md border border-app-line bg-app-box p-1 text-sm shadow-md">
  <div className="px-2 py-1 text-ink/50">搜索类型</div>
  <div
    className="flex items-center justify-start rounded-md px-2 py-2 text-ink hover:bg-app-hover"
    onClick={() => handleSearch(keywordTyping, 'Frame')}
  >
    <span className="text-ink/50">
      <Icon.image className="w-4" />
    </span>
    <span className="mx-2">搜索视频内容</span>
    <strong>{keywordTyping}</strong>
  </div>
  <div
    className="flex items-center justify-start rounded-md px-2 py-2 text-ink hover:bg-app-hover"
    onClick={() => handleSearch(keywordTyping, 'Transcript')}
  >
    <span className="text-ink/50">
      <Icon.microphone className="w-4" />
    </span>
    <span className="mx-2">搜索视频语音</span>
    <strong>{keywordTyping}</strong>
  </div>
</div>
*/
