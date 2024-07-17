import { createContext, useContext } from 'react'

export interface ExplorerApiContextProps {
  listApi: 'assets.list' | 'assets.trash'
  moveApi: 'assets.move_file_path' | 'assets.move_trash_file_path'
}

const ExplorerApiContext = createContext<ExplorerApiContextProps | null>(null)

export const useExplorerApiContext = () => {
  const ctx = useContext(ExplorerApiContext)
  if (ctx === null) throw new Error('ExplorerApiContext.Provider not found!')
  return ctx
}

export const ExplorerApiContextProvider = ExplorerApiContext.Provider
