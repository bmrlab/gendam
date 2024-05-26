'use client'
import ExplorerLayout from '@/Explorer/components/ExplorerLayout'
import { ExplorerContextProvider, ExplorerViewContextProvider, useExplorerValue } from '@/Explorer/hooks'
import { ExplorerItem } from '@/Explorer/types'
import PageNav from '@/components/PageNav'
import Viewport from '@/components/Viewport'
import { Video_Files } from '@gendam/assets/images'
import { Checkbox } from '@gendam/ui/v2/checkbox'
import classNames from 'classnames'
import Image from 'next/image'
import { useCallback, useEffect, useRef, useState } from 'react'
import SearchForm, { type SearchFormRef } from './SearchForm'
import SearchItemContextMenu from './SearchItemContextMenu'
import SearchResults from './SearchResults'
import SearchSuggestions from './SearchSuggestions'
import { SearchPageContextProvider, useSearchPageContext, type SearchResultPayload } from './context'

function SearchPage() {
  const searchQuery = useSearchPageContext()
  const { requestPayload } = searchQuery

  const searchFormRef = useRef<SearchFormRef>(null)
  const onSearchFormSubmit = useCallback(() => {
    const value = searchFormRef.current?.getValue()
    if (value?.text && value?.recordType) {
      searchQuery.fetch({
        api: 'search.all',
        text: value.text,
        recordType: value.recordType,
      })
    } else {
      searchQuery.fetch(null)
    }
  }, [searchQuery])
  const handleSearch = useCallback(
    (text: string, recordType: 'Frame' | 'Transcript') => {
      if (searchFormRef.current) {
        searchFormRef.current.setValue({ text, recordType })
        searchQuery.fetch({
          api: 'search.all',
          text,
          recordType,
        })
      }
    },
    [searchQuery],
  )
  useEffect(() => {
    if (searchFormRef.current && requestPayload?.api === 'search.all') {
      searchFormRef.current.setValue(requestPayload)
    }
  }, [requestPayload])

  const [groupFrames, setGroupFrames] = useState(false)
  const [items, setItems] = useState<SearchResultPayload[] | null>(null)
  const explorer = useExplorerValue({
    items: items
      ? items.map((item) => ({
          type: 'SearchResult',
          filePath: item.filePath,
          metadata: item.metadata,
        }))
      : null,
    settings: {
      layout: 'grid',
    },
  })

  const resetSelectedItems = explorer.resetSelectedItems
  useEffect(() => {
    if (searchQuery.isSuccess) {
      setItems([...searchQuery.data])
      // 重新获取数据要清空选中的项目，以免出现不在列表中但是还被选中的情况
      resetSelectedItems()
    }
  }, [searchQuery.isSuccess, searchQuery.data, resetSelectedItems])

  const contextMenu = (data: ExplorerItem) => {
    return data.type === 'SearchResult' ? (
      <SearchItemContextMenu
        data={{
          filePath: data.filePath,
          metadata: data.metadata,
        }}
      />
    ) : null
  }

  const ToolBar =
    requestPayload?.api === 'search.all' ? (
      <div className="border-app-line flex items-center justify-start border-b px-8 py-2">
        <div className="border-app-line flex items-center overflow-hidden rounded-lg border text-xs">
          <div
            className={classNames('px-4 py-2', requestPayload.recordType === 'Frame' && 'bg-app-hover')}
            onClick={() => handleSearch(requestPayload.text, 'Frame')}
          >
            Visual
          </div>
          <div
            className={classNames('px-4 py-2', requestPayload.recordType === 'Transcript' && 'bg-app-hover')}
            onClick={() => handleSearch(requestPayload.text, 'Transcript')}
          >
            Transcript
          </div>
        </div>
        {/* <div className="text-ink/50 ml-4 text-sm flex-1 truncate">{requestPayload.text}</div> */}
        <form className="ml-auto mr-3 flex items-center gap-2">
          <Checkbox.Root
            id="--group-frames"
            checked={groupFrames}
            onCheckedChange={(checked: boolean | 'indeterminate') => {
              setGroupFrames(checked === true ? true : false)
            }}
          >
            <Checkbox.Indicator />
          </Checkbox.Root>
          <label className="text-xs" htmlFor="--group-frames">
            Expand video frames
          </label>
        </form>
      </div>
    ) : null

  return (
    <Viewport.Page>
      <Viewport.Toolbar className="relative">
        <PageNav
          title={requestPayload?.api === 'search.all' ? `Searching "${requestPayload.text}"` : 'Search'}
          className="max-w-[25%] overflow-hidden"
        />
        <div className="absolute left-1/3 w-1/3">
          <SearchForm ref={searchFormRef} onSubmit={() => onSearchFormSubmit()} />
        </div>
      </Viewport.Toolbar>
      <Viewport.Content className="flex flex-col items-stretch">
        {ToolBar}
        {!requestPayload ? (
          <div className="flex flex-1 flex-col items-center justify-center">
            <Image src={Video_Files} alt="video files" priority className="h-60 w-60"></Image>
            <div className="my-4 text-sm">Search for visual objects or processed transcripts</div>
            <div className="mb-2 text-sm">Try searching for:</div>
            <SearchSuggestions onSelectText={(text) => handleSearch(text, 'Frame')} />
          </div>
        ) : searchQuery.isLoading ? (
          <div className="text-ink/50 flex flex-1 items-center justify-center px-2 py-8 text-sm">Searching...</div>
        ) : searchQuery.isSuccess && searchQuery.data.length === 0 ? (
          <div className="flex flex-1 flex-col items-center justify-center">
            <Image src={Video_Files} alt="video files" priority className="h-60 w-60"></Image>
            <div className="my-4 text-sm">No results found</div>
          </div>
        ) : searchQuery.isSuccess && searchQuery.data.length > 0 ? (
          <ExplorerViewContextProvider value={{ contextMenu }}>
            <ExplorerContextProvider explorer={explorer}>
              <ExplorerLayout
                className="flex-1 p-8"
                renderLayout={() => <SearchResults groupFrames={groupFrames} />}
              ></ExplorerLayout>
            </ExplorerContextProvider>
          </ExplorerViewContextProvider>
        ) : (
          <div className="text-ink/50 flex flex-1 items-center justify-center">Something went wrong</div>
        )}
      </Viewport.Content>
    </Viewport.Page>
  )
}

export default function Search() {
  return (
    <SearchPageContextProvider>
      <SearchPage />
    </SearchPageContextProvider>
  )
}
