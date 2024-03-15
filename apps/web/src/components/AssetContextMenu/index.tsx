'use client'
import { useExplorerContext } from '@/components/Explorer/Context'
import { FilePathWithAssetObject, useAssetContextMenu } from './Context'
import { PropsWithChildren } from 'react'

export default function AssetContextMenu({ children, item }: PropsWithChildren<{
  item: FilePathWithAssetObject
}>) {
  const explorer = useExplorerContext()
  const assetContextMenu = useAssetContextMenu()

	return (
		<div
      onDoubleClick={(e) => {
        // e.stopPropagation()
        assetContextMenu.onDoubleClick(item)
        explorer.resetSelectedItems()
      }}
      onContextMenu={(e) => {
        e.preventDefault()
        assetContextMenu.onContextMenu(item)
        explorer.resetSelectedItems()
      }}
    >
      {children}
    </div>
	);
};
