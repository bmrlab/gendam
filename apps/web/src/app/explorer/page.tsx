'use client'
import ExplorerLayout from '@/Explorer/components/ExplorerLayout'
import {
  ExplorerContextProvider,
  ExplorerViewContextProvider,
  useExplorerValue,
  type ExplorerValue,
} from '@/Explorer/hooks'
// import { useExplorerStore } from '@/Explorer/store'
import { type ExtractExplorerItem } from '@/Explorer/types'
import Inspector from '@/components/Inspector'
import { useInspector } from '@/components/Inspector/store'
import AudioDialog from '@/components/TranscriptExport/AudioDialog'
import Viewport from '@/components/Viewport'
import { FilePath } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { Drop_To_Folder } from '@gendam/assets/images'
import { RSPCError } from '@rspc/client'
import { AnimatePresence, LayoutGroup, motion } from 'framer-motion'
import Image from 'next/image'
import { useSearchParams } from 'next/navigation'
import { useEffect, useMemo, useState } from 'react'
import Footer from './_components/Footer'
import Header from './_components/Header'
import ItemContextMenuV2 from './_components/ItemContextMenu'
import { useResizableInspector } from './_hooks/inspector'

export default function ExplorerPage() {
  // const explorerStore = useExplorerStore()
  // const currentLibrary = useCurrentLibrary()
  const searchParams = useSearchParams()
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
    items: items ? items.map((item) => ({ type: 'FilePath', filePath: item, assetObject: item.assetObject! })) : null,
    materializedPath,
    settings: {
      layout,
    },
  })

  const assetsQueryParams = {
    materializedPath,
    includeSubDirs: layout === 'media' ? true : false,
  }
  const assetsQuery = rspc.useQuery(['assets.list', assetsQueryParams], {
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

  type T = ExtractExplorerItem<'FilePath'>
  const inspectorItem = useMemo<T | null>(() => {
    const selectedItems = Array.from(explorer.selectedItems).filter((item) => item.type === 'FilePath') as T[]
    if (selectedItems.length === 1) {
      return selectedItems[0]
    }
    return null
  }, [explorer.selectedItems])

  /**
   * listen to meta + I to toggle inspector
   * @todo 这个快捷键目前只是临时实现，之后应该统一的管理快捷键并且提供用户自定义的功能
   */
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.metaKey && e.key === 'i') {
        inspector.setShow(!inspector.show)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [inspector])

  /**
   * TODO Inspector 拖拽的时候会有一些性能问题，看起来有点卡
   * 如果拖拽的时候不显示 ExplorerLayout 会好很多
   */
  const { handleRef, width, isResizing } = useResizableInspector()

  if (assetsQuery.isError) {
    return <Viewport.Page className="text-ink/50 flex items-center justify-center">Failed to load assets</Viewport.Page>
  }

  return (
    <ExplorerViewContextProvider
      value={{
        contextMenu: (data) => (data.type === 'FilePath' ? <ItemContextMenuV2 /> : null),
      }}
    >
      <ExplorerContextProvider explorer={explorer}>
        <Viewport.Page>
          <Header /* Viewport.Toolbar */ />
          {assetsQuery.isSuccess && assetsQuery.data.length === 0 ? (
            <Viewport.Content className="flex flex-col items-center justify-center">
              <Image src={Drop_To_Folder} alt="drop to folder" priority className="h-60 w-60"></Image>
              <div className="my-4 text-sm">Drag or paste videos here</div>
            </Viewport.Content>
          ) : (
            <Viewport.Content className="flex h-full w-full overflow-hidden">
              <LayoutGroup>
                <motion.div
                  className="h-full"
                  animate={{
                    width: inspector.show ? `calc(100% - ${width}px)` : '100%',
                  }}
                  transition={
                    isResizing
                      ? {
                          type: 'spring',
                          duration: 0,
                        }
                      : {
                          type: 'spring',
                          stiffness: 500,
                          damping: 50,
                        }
                  }
                >
                  <ExplorerLayout className="h-full w-full overflow-scroll" />
                </motion.div>
                <AnimatePresence mode="popLayout">
                  {inspector.show && (
                    <motion.div
                      layout
                      initial={{
                        x: '100%',
                      }}
                      animate={{
                        x: 0,
                      }}
                      exit={{
                        x: '100%',
                      }}
                      transition={{
                        x: {
                          type: 'spring',
                          stiffness: 500,
                          damping: 50,
                        },
                      }}
                      style={{ width }}
                      className="flex h-full flex-none"
                    >
                      <Inspector data={inspectorItem} ref={handleRef} />
                    </motion.div>
                  )}
                </AnimatePresence>
              </LayoutGroup>
            </Viewport.Content>
          )}
          <Footer /* Viewport.StatusBar */ />
        </Viewport.Page>
        <AudioDialog />
      </ExplorerContextProvider>
    </ExplorerViewContextProvider>
  )
}
