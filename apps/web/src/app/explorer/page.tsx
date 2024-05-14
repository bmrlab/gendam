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
import AudioDialog from '@/components/Audio/AudioDialog'
import { rspc } from '@/lib/rspc'
import { Drop_To_Folder } from '@gendam/assets/images'
import { RSPCError } from '@rspc/client'
import Image from 'next/image'
import { useSearchParams } from 'next/navigation'
import { useEffect, useMemo, useState } from 'react'
import Inspector from '@/components/Inspector'
import Footer from './_components/Footer'
import Header from './_components/Header'
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

  const inspectorItem = useMemo<ExplorerItem | null>(() => {
    const selectedItems = explorer.selectedItems
    if (selectedItems.size === 1) {
      return Array.from(selectedItems)[0]
    }
    return null
  }, [explorer.selectedItems])

  const contextMenu = (data: ExplorerItem) => <ItemContextMenu data={data} />

  if (assetsQuery.isError) {
    return <Viewport.Page className="text-ink/50 flex items-center justify-center">Failed to load assets</Viewport.Page>
  }

  return (
    <ExplorerViewContextProvider value={{ contextMenu }}>
      <ExplorerContextProvider explorer={explorer}>
        <Viewport.Page>
          <Header /* Viewport.Toolbar */ />
          {assetsQuery.isSuccess && assetsQuery.data.length === 0 ? (
            <Viewport.Content className="flex flex-col items-center justify-center">
              <Image src={Drop_To_Folder} alt="drop to folder" priority className="h-60 w-60"></Image>
              <div className="my-4 text-sm">Drag or paste videos here</div>
            </Viewport.Content>
          ) : (
            <Viewport.Content className="h-full flex flex-row overflow-hidden">
              <ExplorerLayout className="h-full flex-1 w-auto overflow-scroll" />
              <Inspector data={inspectorItem} />
            </Viewport.Content>
          )}
          <Footer /* Viewport.StatusBar */ />
        </Viewport.Page>
        <AudioDialog />
      </ExplorerContextProvider>
    </ExplorerViewContextProvider>
  )
}
