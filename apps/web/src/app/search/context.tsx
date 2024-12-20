import type {
  // RecommendRequestPayload,
  // SearchRequestPayload,
  SearchResultData,
} from '@/lib/bindings'
import { client } from '@/lib/rspc'
import { useSearchParams } from 'next/navigation'
import {
  ContextType,
  PropsWithChildren,
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
} from 'react'

export type SearchPayload =
  | {
      api: 'search.all'
      text: string
    }
  | {
      api: 'search.recommend'
      filePath?: SearchResultData['filePath'] // FilePath
      assetObjectHash: string
      timestamp: number
    }

export { type SearchResultData } from '@/lib/bindings'

function useSearchPayloadInURL(): {
  searchPayloadInURL: SearchPayload | null
  updateSearchPayloadInURL: (payload: SearchPayload | null) => void
} {
  const searchParams = useSearchParams()
  const searchPayloadInURL = useMemo<Extract<SearchPayload, { api: 'search.all' }> | null>(() => {
    try {
      const text = searchParams.get('text')
      // const recordType = searchParams.get('recordType')
      // if (text && (recordType === 'Frame' || recordType === 'Transcript')) {
      //   return { api: 'search.all', text, recordType }
      // }
      if (text) return { api: 'search.all', text }
    } catch (e) {}
    return null
  }, [searchParams])

  const updateSearchPayloadInURL = useCallback((payload: SearchPayload | null) => {
    if (payload?.api === 'search.all') {
      const search = new URLSearchParams()
      search.set('text', payload.text)
      window.history.replaceState({}, '', `${window.location.pathname}?${search}`)
    } else {
      window.history.replaceState({}, '', `${window.location.pathname}`)
    }
  }, [])

  return { searchPayloadInURL, updateSearchPayloadInURL }
}

type TSearchPageContext = {
  requestPayload: SearchPayload | null
  data: SearchResultData[]
  fetch: (payload: SearchPayload | null) => void
  isLoading: boolean
  isSuccess: boolean
}
const SearchPageContext = createContext<TSearchPageContext | null>(null)

export const useSearchPageContext = () => {
  const ctx = useContext(SearchPageContext)
  return ctx as NonNullable<ContextType<typeof SearchPageContext>>
}

export function SearchPageContextProvider({ children }: PropsWithChildren<{}>) {
  const { searchPayloadInURL, updateSearchPayloadInURL } = useSearchPayloadInURL()

  const [requestPayload, setRequestPayload] = useState<SearchPayload | null>(searchPayloadInURL)
  const [isLoading, setIsLoading] = useState(false)
  const [isSuccess, setIsSuccess] = useState(false)
  const [data, setData] = useState<SearchResultData[]>([])

  const fetch = useCallback(
    async (payload: SearchPayload | null) => {
      updateSearchPayloadInURL(payload)
      setRequestPayload(payload)
      if (!payload) {
        setData([])
        return
      }
      setIsLoading(true)
      try {
        const res = await client.query([payload.api, payload])
        setData(res)
        setIsSuccess(true)
      } catch (e) {
        setData([])
        setIsSuccess(false)
      }
      setIsLoading(false)
    },
    [updateSearchPayloadInURL],
  )

  const [initialized, setInitialized] = useState(false)
  useEffect(() => {
    if (initialized) {
      return
    }
    setInitialized(true)
    if (requestPayload) {
      fetch(requestPayload)
    }
  }, [fetch, requestPayload, initialized])

  return (
    <SearchPageContext.Provider
      value={{
        requestPayload,
        data,
        fetch,
        isLoading,
        isSuccess,
      }}
    >
      {children}
    </SearchPageContext.Provider>
  )
}
