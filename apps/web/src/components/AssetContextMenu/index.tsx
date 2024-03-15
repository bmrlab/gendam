'use client'
import { MenuProps, AssetContextMenuProvider } from './Context'
import { PropsWithChildren } from 'react'

export default function AssetContextMenu({
  children,
  ...menuProps
}: PropsWithChildren<MenuProps>) {
	return (
		<AssetContextMenuProvider {...menuProps}>
      {children}
		</AssetContextMenuProvider>
	);
};
