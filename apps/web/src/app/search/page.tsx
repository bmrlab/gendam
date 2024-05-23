'use client'
import PageNav from '@/components/PageNav'
import Viewport from '@/components/Viewport'
import type { SearchRequestPayload, SearchResultPayload } from '@/lib/bindings'
import { Video_Files } from '@gendam/assets/images'
import Image from 'next/image'
import { rspc } from '@/lib/rspc'
import classNames from 'classnames'
import { useSearchParams } from 'next/navigation'
import React, { useCallback, useEffect, useMemo, useState } from 'react'
import SearchForm from './SearchForm'
import SearchResults from './SearchResults'
import { Checkbox } from '@gendam/ui/v2/checkbox'
import SearchItemContextMenu from './SearchItemContextMenu'
import ExplorerLayout from '@/Explorer/components/ExplorerLayout'
import {
  ExplorerContextProvider,
  ExplorerViewContextProvider,
  useExplorerValue,
} from '@/Explorer/hooks'
import { ExplorerItem } from '@/Explorer/types'

const useSearchPayloadInURL: () => [
  SearchRequestPayload | null,
  (payload: SearchRequestPayload) => void,
] = () => {
  const searchParams = useSearchParams()
  const searchPayloadInURL = useMemo<SearchRequestPayload | null>(() => {
    try {
      const text = searchParams.get('text')
      const recordType = searchParams.get('recordType')
      if (text && recordType) {
        return { text, recordType }
      }
    } catch (e) {}
    return null
  }, [searchParams])

  const updateSearchPayloadInURL = useCallback((payload: SearchRequestPayload) => {
    const search = new URLSearchParams()
    search.set('text', payload.text)
    search.set('recordType', payload.recordType)
    window.history.replaceState({}, '', `${window.location.pathname}?${search}`)
  }, [])

  return [searchPayloadInURL, updateSearchPayloadInURL]
}

export default function Search() {
  const [searchPayloadInURL, updateSearchPayloadInURL] = useSearchPayloadInURL()
  const [searchPayload, setSearchPayload] = useState<SearchRequestPayload | null>(searchPayloadInURL)
  const queryRes = rspc.useQuery(['search.all', searchPayload!], {
    enabled: !!searchPayload,
  })
  const [groupFrames, setGroupFrames] = useState(false)
  const [items, setItems] = useState<SearchResultPayload[] | null>(null)
  const suggestionsQuery = rspc.useQuery(['search.suggestions'])

  const pickedSuggestions = useMemo(() => {
    // shuffle pick 5 suggestions
    if (suggestionsQuery.data) {
      const suggestions = [...suggestionsQuery.data]
      const picked = []
      while (picked.length < 5 && suggestions.length > 0) {
        const index = Math.floor(Math.random() * suggestions.length)
        picked.push(suggestions[index])
        suggestions.splice(index, 1)
      }
      return picked
    } else {
      return []
    }
  }, [suggestionsQuery.data])

  const handleSearch = useCallback(
    (text: string, recordType: string) => {
      if (text && recordType) {
        const payload = { text, recordType }
        setSearchPayload(payload)
        updateSearchPayloadInURL(payload)
      }
    },
    [setSearchPayload, updateSearchPayloadInURL],
  )

  const explorer = useExplorerValue({
    items: items ? items.map((item) => ({
      type: 'SearchResult',
      filePath: item.filePath,
      metadata: item.metadata,
    })) : null,
    settings: {
      layout: 'grid',
    },
  })

  const resetSelectedItems = explorer.resetSelectedItems
  useEffect(() => {
    if (queryRes.isSuccess) {
      setItems([...queryRes.data])
      // 重新获取数据要清空选中的项目，以免出现不在列表中但是还被选中的情况
      resetSelectedItems()
    }
  }, [queryRes.isSuccess, queryRes.data, resetSelectedItems])

  const renderLayout = () => {
    if (!queryRes.data) {
      return <></>
    }
    return <SearchResults groupFrames={groupFrames} />
  }

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

  return (
    <Viewport.Page>
      <Viewport.Toolbar className="relative">
        <PageNav
          title={searchPayload ? `Searching "${searchPayload.text}"` : "Search"}
          className="max-w-[25%] overflow-hidden"
        />
        <div className="absolute left-1/3 w-1/3">
          <SearchForm
            initialSearchPayload={searchPayloadInURL}
            onSubmit={(text: string, recordType: string) => handleSearch(text, recordType)}
          />
        </div>
      </Viewport.Toolbar>
      <Viewport.Content className="flex flex-col items-stretch">
        {searchPayload ? (
          <div className="border-app-line flex items-center justify-start border-b px-8 py-2">
            <div className="border-app-line flex items-center overflow-hidden rounded-lg border text-xs">
              <div
                className={classNames('px-4 py-2', searchPayload.recordType === 'Frame' && 'bg-app-hover')}
                onClick={() => handleSearch(searchPayload.text, 'Frame')}
              >Visual</div>
              <div
                className={classNames('px-4 py-2', searchPayload.recordType === 'Transcript' && 'bg-app-hover')}
                onClick={() => handleSearch(searchPayload.text, 'Transcript')}
              >Transcript</div>
            </div>
            {/* <div className="text-ink/50 ml-4 text-sm flex-1 truncate">{searchPayload.text}</div> */}
            <form className='ml-auto flex items-center gap-2 mr-3'>
              <Checkbox.Root
                id="--group-frames" checked={groupFrames}
                onCheckedChange={(checked: boolean | 'indeterminate') => {
                  setGroupFrames(checked === true ? true : false)
                }}
              >
                <Checkbox.Indicator />
              </Checkbox.Root>
              <label className="text-xs" htmlFor="--group-frames">Expand video frames</label>
            </form>
          </div>
        ) : null}

        {!searchPayload ? (
          <div className="flex-1 flex flex-col items-center justify-center">
            <Image src={Video_Files} alt="video files" priority className="w-60 h-60"></Image>
            <div className="my-4 text-sm">Search for visual objects or processed transcripts</div>
            <div className="mb-2 text-sm">Try searching for:</div>
            <div className="mb-4 text-ink/70 text-xs">
              {pickedSuggestions.map((suggestion, index) => (
                <div
                  key={index} className="py-1 text-center hover:underline"
                  onClick={() => handleSearch(suggestion, 'Frame')}
                >
                  &quot;{ suggestion }&quot;
                </div>
              ))}
            </div>
          </div>
        ) : queryRes.isLoading ? (
          <div className="flex-1 text-ink/50 flex items-center justify-center px-2 py-8 text-sm">Searching...</div>
        ) : queryRes.isSuccess && queryRes.data.length === 0 ? (
          <div className="flex-1 flex flex-col items-center justify-center">
            <Image src={Video_Files} alt="video files" priority className="w-60 h-60"></Image>
            <div className="my-4 text-sm">
              No results found for <span className="font-medium">{searchPayload.text}</span>
            </div>
          </div>
        ) : queryRes.isSuccess && queryRes.data.length > 0 ? (
          <ExplorerViewContextProvider value={{ contextMenu }}>
            <ExplorerContextProvider explorer={explorer}>
              <ExplorerLayout
                className="flex-1 p-8"
                renderLayout={renderLayout}
              ></ExplorerLayout>
            </ExplorerContextProvider>
          </ExplorerViewContextProvider>
        ) : (
          <div className="flex-1 text-ink/50 flex items-center justify-center">Something went wrong</div>
        )}
      </Viewport.Content>
    </Viewport.Page>
  )
}
