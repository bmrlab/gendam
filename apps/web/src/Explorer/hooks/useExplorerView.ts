// see spacedrive's interface/app/$libraryId/Explorer/View/Context.ts
import { createContext, useContext, type ReactNode } from 'react'
import { ExplorerItem } from '../types'

export interface ExplorerViewContextProps {
  // ref: RefObject<HTMLDivElement>
  contextMenu?: (data: ExplorerItem) => ReactNode
}

const ExplorerViewContext = createContext<ExplorerViewContextProps | null>(null)

export const useExplorerViewContext = () => {
  const ctx = useContext(ExplorerViewContext)
  if (ctx === null) throw new Error('ExplorerViewContext.Provider not found!')
  return ctx
}

export const ExplorerViewContextProvider = ExplorerViewContext.Provider
