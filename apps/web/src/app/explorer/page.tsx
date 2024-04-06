'use client'
import ExplorerLayout from '@/Explorer/components/ExplorerLayout'
import { ExplorerContextProvider, ExplorerViewContextProvider, useExplorer } from '@/Explorer/hooks'
// import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import Viewport from '@/components/Viewport'
import { rspc } from '@/lib/rspc'
import { RSPCError } from '@rspc/client'
import { useSearchParams } from 'next/navigation'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { FoldersDialog } from './_components/FoldersDialog'
import Footer from './_components/Footer'
import Header from './_components/Header'
import Inspector from './_components/Inspector'
import ItemContextMenu from './_components/ItemContextMenu'

export default function ExplorerPage() {
  // const explorerStore = useExplorerStore()
  const searchParams = useSearchParams()
  let dirInSearchParams = searchParams.get('dir') || '/'
  if (!/^\/([^/\\:*?"<>|]+\/)+$/.test(dirInSearchParams)) {
    dirInSearchParams = '/'
  }
  // currentPath 必须以 / 结尾, 调用 setCurrentPath 的地方自行确保格式正确
  // const [parentPath, setParentPath] = useState<string>(dirInSearchParams)
  const parentPath = useMemo(() => dirInSearchParams, [dirInSearchParams])
  const moveMut = rspc.useMutation(['assets.move_file_path'])
  const [items, setItems] = useState<ExplorerItem[] | null>(null)

  const explorer = useExplorer({
    items: items,
    parentPath: parentPath,
    settings: {
      layout: 'grid',
    },
  })

  const assetsQuery = rspc.useQuery(
    [
      'assets.list',
      {
        materializedPath: parentPath,
        includeSubDirs: explorer.settings.layout === 'media' ? true : false,
      },
    ],
    {
      /**
       * 这样可以在删除/重命名/刷新metadata等操作执行以后自动刷新
       * 但现在看起来虽然全局设置了 refetchOnWindowFocus: false, 还是会自动刷新的
       */
      // refetchOnWindowFocus: true,
      throwOnError: (e: RSPCError) => {
        console.log(e)
        return false // stop propagate throwing error
      },
    },
  )

  useEffect(() => {
    if (assetsQuery.isSuccess) {
      setItems([ ...assetsQuery.data ])
    }
  }, [assetsQuery.isSuccess, assetsQuery.data, setItems])

  const contextMenu = (data: ExplorerItem) => <ItemContextMenu data={data} />

  const onMoveTargetSelected = useCallback(
    (target: ExplorerItem | null) => {
      for (let active of Array.from(explorer.selectedItems)) {
        // target 可以为空，为空就是根目录，这时候不需要检查 target.id !== active.id，因为根目录本身不会被移动
        if (!target || target.id !== active.id) {
          moveMut.mutate({
            active: {
              id: active.id,
              materializedPath: active.materializedPath,
              isDir: active.isDir,
              name: active.name,
            },
            target: target
              ? {
                  id: target.id,
                  materializedPath: target.materializedPath,
                  isDir: target.isDir,
                  name: target.name,
                }
              : null,
          })
        }
      }
    },
    [explorer, moveMut],
  )

  if (assetsQuery.isError) {
    return <Viewport.Page className="flex items-center justify-center text-ink/50">Failed to load assets</Viewport.Page>
  }

  return (
    <ExplorerViewContextProvider value={{ contextMenu }}>
      <ExplorerContextProvider explorer={explorer}>
        <Viewport.Page onClick={() => explorer.resetSelectedItems()}>
          <Header />

          <div className="flex-1 w-full flex flex-row overflow-hidden">
            <Viewport.Content className="w-auto">
              <ExplorerLayout></ExplorerLayout>
            </Viewport.Content>
            <Inspector />
          </div>

          <Footer />
          <FoldersDialog onConfirm={onMoveTargetSelected} />
        </Viewport.Page>
      </ExplorerContextProvider>
    </ExplorerViewContextProvider>
  )
}
