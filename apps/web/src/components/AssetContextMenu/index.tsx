'use client'
import { FilePathWithAssetObject, useAssetContextMenu } from './Context'
import { PropsWithChildren, useCallback } from 'react'

export default function AssetContextMenu({ children, item }: PropsWithChildren<{
  item: FilePathWithAssetObject
}>) {
  const assetContextMenu = useAssetContextMenu()

  let handleDoubleClick = useCallback((asset: FilePathWithAssetObject) => {
    assetContextMenu.onDoubleClick(asset)
  }, [assetContextMenu])

	return (
		<div onDoubleClick={(e) => {
      // e.stopPropagation()
      handleDoubleClick(item)
    }}>
      {children}
    </div>
	);
};
