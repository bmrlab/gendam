import { createContext, useContext } from 'react'
import { Library as _Library } from '@/lib/bindings'

export type Library = _Library

type CurrentLibraryContext = {
  id?: string
  dir?: string
  settings?: _Library["settings"],
  setContext: (library: Library) => Promise<void>
  getFileSrc: (assetObjectId: string) => string
  getThumbnailSrc: (assetObjectId: string, timestampInSecond?: number) => string
}

export const CurrentLibrary = createContext<CurrentLibraryContext>({
  setContext: async () => {},
  getFileSrc: (assetObjectHash) => `/images/empty.png?${assetObjectHash}`, // 无效的默认值
  getThumbnailSrc: (assetObjectHash, timestampInSecond?) => `/images/empty.png?${assetObjectHash}&${timestampInSecond}`, // 无效的默认值
})

export const useCurrentLibrary = () => useContext(CurrentLibrary)
