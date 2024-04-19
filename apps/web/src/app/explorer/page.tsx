'use client'
import ExplorerLayout from '@/Explorer/components/ExplorerLayout'
import {
  ExplorerContextProvider,
  ExplorerViewContextProvider,
  useExplorerValue,
  type ExplorerValue,
} from '@/Explorer/hooks'
// import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import Viewport from '@/components/Viewport'
import { queryClient, rspc } from '@/lib/rspc'
import { Drop_To_Folder } from '@muse/assets/images'
import { RSPCError } from '@rspc/client'
import Image from 'next/image'
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
  // const [materializedPath, setMaterializedPath] = useState<string>(dirInSearchParams)
  const materializedPath = useMemo(() => dirInSearchParams, [dirInSearchParams])
  const moveMut = rspc.useMutation(['assets.move_file_path'])
  const [items, setItems] = useState<ExplorerItem[] | null>(null)
  const [layout, setLayout] = useState<ExplorerValue['settings']['layout']>('grid')

  const assetsQuery = rspc.useQuery(
    [
      'assets.list',
      {
        materializedPath: materializedPath,
        includeSubDirs: layout === 'media' ? true : false,
      },
    ],
    {
      // refetchOnWindowFocus: true,
      throwOnError: (e: RSPCError) => {
        console.log(e)
        return false // stop propagate throwing error
      },
    },
  )

  const explorer = useExplorerValue({
    items: items,
    materializedPath: materializedPath,
    settings: {
      layout,
    },
  })

  const resetSelectedItems = explorer.resetSelectedItems
  useEffect(() => {
    if (assetsQuery.isSuccess) {
      setItems([...assetsQuery.data])
      // 重新获取数据要清空选中的项目，以免出现不在列表中但是还被选中的情况
      resetSelectedItems()
    }
  }, [assetsQuery.isLoading, assetsQuery.isSuccess, assetsQuery.data, resetSelectedItems])

  useEffect(() => {
    setLayout(explorer.settings.layout)
  }, [explorer.settings.layout])

  const contextMenu = (data: ExplorerItem) => <ItemContextMenu data={data} />

  const onMoveTargetSelected = useCallback(
    async (target: ExplorerItem | null) => {
      for (let active of Array.from(explorer.selectedItems)) {
        // target 可以为空，为空就是根目录，这时候不需要检查 target.id !== active.id，因为根目录本身不会被移动
        if (target && target.id === active.id) {
          continue
        }
        try {
          await moveMut.mutateAsync({
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
        } catch (error) {}
        queryClient.invalidateQueries({
          queryKey: ['assets.list', { materializedPath: materializedPath }],
        })
        queryClient.invalidateQueries({
          queryKey: [
            'assets.list',
            {
              materializedPath: target ? target.materializedPath + target.name + '/' : '/',
            },
          ],
        })
      }
    },
    [explorer.selectedItems, materializedPath, moveMut],
  )

  if (assetsQuery.isError) {
    return <Viewport.Page className="text-ink/50 flex items-center justify-center">Failed to load assets</Viewport.Page>
  }

  return (
    <ExplorerViewContextProvider value={{ contextMenu }}>
      <ExplorerContextProvider explorer={explorer}>
        <Viewport.Page>
          <Header />

          {assetsQuery.isSuccess && assetsQuery.data.length === 0 ? (
            <Viewport.Content className="flex flex-col items-center justify-center">
              <Image src={Drop_To_Folder} alt="drop to folder" priority className="h-60 w-60"></Image>
              <div className="my-4 text-sm">Drag or paste videos here</div>
            </Viewport.Content>
          ) : (
            <Viewport.Content className="flex flex-row overflow-hidden" onClick={() => explorer.resetSelectedItems()}>
              <ExplorerLayout className="flex-1 w-auto overflow-scroll"></ExplorerLayout>
              <Inspector />
            </Viewport.Content>
          )}

          <Footer />
          <FoldersDialog onConfirm={onMoveTargetSelected} />
        </Viewport.Page>
      </ExplorerContextProvider>
    </ExplorerViewContextProvider>
  )
}
