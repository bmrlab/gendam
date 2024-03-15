'use client'
import type { FilePathWithAssetObject } from '@/components/AssetContextMenu/Context'
import { AssetContextMenuProvider } from '@/components/AssetContextMenu/Context'
import Explorer from '@/components/Explorer'
import { ExplorerContextProvider } from '@/components/Explorer/Context'
import { useExplorer } from '@/components/Explorer/useExplorer'
import { CurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { useRouter } from 'next/navigation'
import { useCallback, useContext, useState } from 'react'
import Header from './_components/Header'

export default function ExplorerPage() {
  const currentLibrary = useContext(CurrentLibrary)
  const router = useRouter()
  // currentPath 必须以 / 结尾, 调用 setCurrentPath 的地方自行确保格式正确
  const [parentPath, setParentPath] = useState<string>('/')

  const processVideoMut = rspc.useMutation(['assets.process_video_asset'])

  const {
    data: assets,
    isLoading,
    error,
  } = rspc.useQuery([
    'assets.list',
    {
      path: parentPath,
      dirsOnly: false,
    },
  ])

  const explorer = useExplorer({
    items: assets ?? null,
    parentPath: parentPath,
    settings: {
      layout: 'grid',
    },
  })

  const goToDir = useCallback(
    (dirName: string) => {
      let newPath = parentPath
      if (dirName === '-1') {
        newPath = newPath.replace(/(.*\/)[^/]+\/$/, '$1')
      } else {
        newPath += dirName + '/'
      }
      setParentPath(newPath)
    },
    [setParentPath, parentPath],
  )

  let handleDoubleClick = useCallback(
    (asset: FilePathWithAssetObject) => {
      if (asset.isDir) {
        goToDir(asset.name)
      } else {
        // this will always be true if asset.isDir is false
        // revealMut.mutate("/" + asset.assetObject.id.toString());
        processVideoMut.mutate(asset.id)
        router.push('/video-tasks')
      }
    },
    [goToDir, processVideoMut, router],
  )

  return (
    <AssetContextMenuProvider onDoubleClick={handleDoubleClick}>
      <ExplorerContextProvider explorer={explorer}>
        <div className="h-full flex flex-col" onClick={() => explorer.resetSelectedItems()}>
          <Header goToDir={goToDir} parentPath={parentPath}></Header>
          <div className='flex-1'>
            <Explorer></Explorer>
          </div>
        </div>
      </ExplorerContextProvider>
    </AssetContextMenuProvider>
  )
}
