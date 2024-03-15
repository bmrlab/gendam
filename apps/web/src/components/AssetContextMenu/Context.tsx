import { createContext, PropsWithChildren, useContext } from 'react'
import { FilePathQueryResult } from '@/lib/bindings'

export type FilePathWithAssetObject = FilePathQueryResult;

export type MenuProps = {
  onDoubleClick: (asset: FilePathWithAssetObject) => void;
}
const Context = createContext<MenuProps>({
  onDoubleClick: () => console.error('AssetContextMenuProvider not found'),
})

export const useAssetContextMenu = () => {
  const ctx = useContext(Context)
  return ctx as MenuProps
}

export function AssetContextMenuProvider<T extends MenuProps>({
  children,
  ...menuProps
}: PropsWithChildren<T>) {
  return <Context.Provider value={menuProps}>{children}</Context.Provider>
}
