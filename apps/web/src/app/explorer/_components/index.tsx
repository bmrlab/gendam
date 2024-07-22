'use client'
import ExplorerLayout from '@/Explorer/components/ExplorerLayout'
import { ExplorerContextProvider, useExplorerValue, type ExplorerValue } from '@/Explorer/hooks'
// import { useExplorerStore } from '@/Explorer/store'
import { type ExplorerItem } from '@/Explorer/types'
import AudioDialog from '@/components/Audio/AudioDialog'
import Inspector from '@/components/Inspector'
import { useInspector } from '@/components/Inspector/store'
import Viewport from '@/components/Viewport'
import { type FilePath } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { Drop_To_Folder } from '@gendam/assets/images'
import { RSPCError } from '@rspc/client'
import Image from 'next/image'
import { useSearchParams } from 'next/navigation'
import { FC, useEffect, useMemo, useState } from 'react'
import Footer from './Footer'
import Header from './Header'
import { useExplorerApiContext } from '@/Explorer/hooks/useExplorerApi'

export const CommonPage: FC = () => {
  // const explorerStore = useExplorerStore()
  const searchParams = useSearchParams()
  const explorerApi = useExplorerApiContext()
  let dirInSearchParams = searchParams.get('dir') || '/'
  if (!/^\/([^/\\:*?"<>|]+\/)+$/.test(dirInSearchParams)) {
    dirInSearchParams = '/'
  }

  // 进入 explorer 页面默认选中的 file path item
  const filePathIdInSearchParams = searchParams.get('id')
  const initialRevealedFilePathId = useMemo(() => {
    return filePathIdInSearchParams ? +filePathIdInSearchParams : null
  }, [filePathIdInSearchParams])
  // const [initialRevealedFilePathId, setInitialRevealedFilePathId] = useState<number | null>(
  //   filePathIdInSearchParams ? +filePathIdInSearchParams : null,
  // )

  // currentPath 必须以 / 结尾, 调用 setCurrentPath 的地方自行确保格式正确
  // const [materializedPath, setMaterializedPath] = useState<string>(dirInSearchParams)
  const materializedPath = useMemo(() => dirInSearchParams, [dirInSearchParams])

  const [items, setItems] = useState<FilePath[] | null>(null)
  const [layout, setLayout] = useState<ExplorerValue['settings']['layout']>('grid')

  const inspector = useInspector()
  const explorer = useExplorerValue({
    items: items ? items.map((item) => ({ type: 'FilePath', filePath: item })) : null,
    materializedPath,
    settings: {
      layout,
    },
  })

  const assetsQueryParams = {
    materializedPath,
    includeSubDirs: layout === 'media' ? true : false,
  }
  const assetsQuery = rspc.useQuery([explorerApi.listApi, assetsQueryParams], {
    // refetchOnWindowFocus: true,
    throwOnError: (e: RSPCError) => {
      console.log(e)
      return false // stop propagate throwing error
    },
  })

  const resetSelectedItems = explorer.resetSelectedItems
  const setShowInspector = inspector.setShow
  useEffect(() => {
    if (assetsQuery.isSuccess) {
      const revealedFilePath = assetsQuery.data.find((item) => item.id === initialRevealedFilePathId)
      setItems([...assetsQuery.data])
      // 重新获取数据要清空选中的项目，以免出现不在列表中但是还被选中的情况
      if (revealedFilePath) {
        resetSelectedItems([
          {
            type: 'FilePath',
            filePath: revealedFilePath,
          },
        ])
        setShowInspector(true)
      } else {
        resetSelectedItems()
      }
    }
  }, [
    assetsQuery.isLoading,
    assetsQuery.isSuccess,
    assetsQuery.data,
    resetSelectedItems,
    initialRevealedFilePathId,
    setShowInspector,
  ])

  useEffect(() => {
    setLayout(explorer.settings.layout)
  }, [explorer.settings.layout])

  const inspectorItem = useMemo<FilePath | null>(() => {
    type T = Extract<ExplorerItem, { type: 'FilePath' }>
    const selectedItems = Array.from(explorer.selectedItems).filter((item) => item.type === 'FilePath') as T[]
    if (selectedItems.length === 1) {
      return selectedItems[0].filePath
    }
    return null
  }, [explorer.selectedItems])

  if (assetsQuery.isError) {
    return <Viewport.Page className="text-ink/50 flex items-center justify-center">Failed to load assets</Viewport.Page>
  }

  return (
    <ExplorerContextProvider explorer={explorer}>
      <Viewport.Page>
        <Header /* Viewport.Toolbar */ />
        {assetsQuery.isSuccess && assetsQuery.data.length === 0 ? (
          <Viewport.Content className="flex flex-col items-center justify-center">
            <Image src={Drop_To_Folder} alt="drop to folder" priority className="h-60 w-60"></Image>
            <div className="my-4 text-sm">Drag or paste videos here</div>
          </Viewport.Content>
        ) : (
          <Viewport.Content className="flex h-full flex-row overflow-hidden">
            <ExplorerLayout className="h-full w-auto flex-1 overflow-scroll" />
            <Inspector data={inspectorItem} />
          </Viewport.Content>
        )}
        <Footer /* Viewport.StatusBar */ />
      </Viewport.Page>
      <AudioDialog />
    </ExplorerContextProvider>
  )
}
