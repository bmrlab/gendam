// TODO: Move to hooks folder
import { ContextType, createContext, PropsWithChildren, useContext } from 'react'
import { type ExplorerValue } from './useExplorerValue'

const ExplorerContext = createContext<ExplorerValue | null>(null)
type ExplorerContext = NonNullable<ContextType<typeof ExplorerContext>>

export const useExplorerContext = () => {
  const ctx = useContext(ExplorerContext)
  return ctx as ExplorerContext
}

export function ExplorerContextProvider<TExplorer extends ExplorerValue>({
  explorer,
  children,
}: PropsWithChildren<{ explorer: TExplorer }>) {
  return <ExplorerContext.Provider value={explorer}>{children}</ExplorerContext.Provider>
}
