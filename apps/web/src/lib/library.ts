import { createContext, useContext } from 'react'

type CurrentLibraryContext = {
  id: string | null
  setContext: (id: string) => Promise<void>
  getFileSrc: (assetObjectId: string) => string
}

export const CurrentLibrary = createContext<CurrentLibraryContext>({
  id: null,
  setContext: async () => {},
  getFileSrc: (assetObjectHash: string) => `http://localhost/${assetObjectHash}`, // 无效的默认值
})

export const useCurrentLibrary = () => useContext(CurrentLibrary)
