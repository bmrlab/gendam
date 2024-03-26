import { createContext, useContext } from 'react'

export type Library = {
  id: string
  settings: { title: string }
}

type CurrentLibraryContext = {
  id: string | null
  settings: {
    title: string,
  } | null,
  setContext: (library: Library) => Promise<void>
  getFileSrc: (assetObjectId: string) => string
}

export const CurrentLibrary = createContext<CurrentLibraryContext>({
  id: null,
  settings: null,
  setContext: async () => {},
  getFileSrc: (assetObjectHash: string) => `http://localhost/${assetObjectHash}`, // 无效的默认值
})

export const useCurrentLibrary = () => useContext(CurrentLibrary)
