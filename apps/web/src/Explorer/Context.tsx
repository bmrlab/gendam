import { ContextType, createContext, PropsWithChildren, useContext } from 'react'
import type { UseExplorer } from './hooks/useExplorer'

const ExplorerContext = createContext<UseExplorer | null>(null)
type ExplorerContext = NonNullable<ContextType<typeof ExplorerContext>>

export const useExplorerContext = () => {
  const ctx = useContext(ExplorerContext)
  return ctx as ExplorerContext
}

export function ExplorerContextProvider<TExplorer extends UseExplorer>({
  explorer,
  children,
}: PropsWithChildren<{ explorer: TExplorer }>) {
  return <ExplorerContext.Provider value={explorer}>{children}</ExplorerContext.Provider>
}
