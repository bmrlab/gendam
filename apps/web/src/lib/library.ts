import { ContextType, createContext, useContext } from 'react'
import { LibrarySettings } from '@/lib/bindings'

export type Library = {
  id: string
  dir: string
}

type CurrentLibraryContext = {
  id: string
  dir: string
  librarySettings: LibrarySettings,
  updateLibrarySettings: (partialSettingssettings: Partial<LibrarySettings>) => Promise<void>,
  getFileSrc: (assetObjectId: string) => string
  getThumbnailSrc: (assetObjectId: string, timestampInSecond?: number) => string
}

export const CurrentLibrary = createContext<CurrentLibraryContext|null>(
  null
  // {
  //   updateLibrarySettings: async () => {},
  //   set: async () => {},
  //   getFileSrc: (assetObjectHash) => `/images/empty.png?${assetObjectHash}`, // 无效的默认值
  //   getThumbnailSrc: (assetObjectHash, timestampInSecond?) => `/images/empty.png?${assetObjectHash}&${timestampInSecond}`, // 无效的默认值
  // }
)

type NonNullableCurrentLibrary = NonNullable<ContextType<typeof CurrentLibrary>>

export const useCurrentLibrary = () => useContext(CurrentLibrary) as NonNullableCurrentLibrary
