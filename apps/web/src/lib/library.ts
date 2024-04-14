import { createContext, useContext } from 'react'

export type Library = {
  id: string
  dir: string
}

type CurrentLibraryContext = {
  id?: string
  dir?: string
  set: (library: Library) => Promise<void>
  getFileSrc: (assetObjectId: string) => string
  getThumbnailSrc: (assetObjectId: string, timestampInSecond?: number) => string
}

export const CurrentLibrary = createContext<CurrentLibraryContext>({
  set: async () => {},
  getFileSrc: (assetObjectHash) => `/images/empty.png?${assetObjectHash}`, // 无效的默认值
  getThumbnailSrc: (assetObjectHash, timestampInSecond?) => `/images/empty.png?${assetObjectHash}&${timestampInSecond}`, // 无效的默认值
})

export const useCurrentLibrary = () => useContext(CurrentLibrary)
