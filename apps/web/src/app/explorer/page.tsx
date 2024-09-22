'use client'
import ExplorerLayout from '@/Explorer/components/ExplorerLayout'
import {
  ExplorerContextProvider,
  ExplorerViewContextProvider,
  useExplorerContext,
  useExplorerValue,
} from '@/Explorer/hooks'
import { useCurrentLibrary } from '@/lib/library'
// import { useExplorerStore } from '@/Explorer/store'
import { InspectorPane, InspectorProvider, useInspector, useResizableInspector } from '@/components/Inspector'
import AudioDialog from '@/components/TranscriptExport/AudioDialog'
import Viewport from '@/components/Viewport'
import { type ExtractExplorerItem } from '@/Explorer/types'
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

function _ExplorerPage() {
  const explorer = useExplorerContext()

  // inspector
  const inspector = useInspector()
  // TODO Inspector 拖拽的时候会有一些性能问题，看起来有点卡，如果拖拽的时候不显示 ExplorerLayout 会好很多
  // explorer 页面上的 inspectorSize 被托管到了 settings 上持久化，这里是设置初始值
  const resizableInspector = useResizableInspector(explorer.settings.inspectorSize)
  useEffect(() => {
    if (resizableInspector.width !== explorer.settings.inspectorSize && !resizableInspector.isResizing) {
      explorer.settings.update({ inspectorSize: resizableInspector.width })
    }
  }, [resizableInspector.width, resizableInspector.isResizing, explorer.settings])

  // listen to meta + I to toggle inspector
  // @todo 这个快捷键目前只是临时实现，之后应该统一的管理快捷键并且提供用户自定义的功能
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.metaKey && e.key === 'i') {
        const show = !inspector.show
        inspector.setShow(show)
        explorer.settings.update({ inspectorShow: show }) // 和 Header 上的按钮一样，会保存设置
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [inspector, explorer.settings])

  // set inspector item data
  type T = ExtractExplorerItem<'FilePathDir'> | ExtractExplorerItem<'FilePathWithAssetObject'>
  const inspectorItemData = useMemo<T | null>(() => {
    const selectedItems = Array.from(explorer.selectedItems).filter(
      (item) => item.type === 'FilePathDir' || item.type === 'FilePathWithAssetObject',
    ) as T[]
    if (selectedItems.length === 1) {
      return selectedItems[0]
    }
    return null
  }, [explorer.selectedItems])

  return (
    <Viewport.Page>
      <Header /* Viewport.Toolbar */ />
      {explorer.items?.length === 0 ? (
        <Viewport.Content className="flex flex-col items-center justify-center">
          <Image src={Drop_To_Folder} alt="drop to folder" priority className="h-60 w-60"></Image>
          <div className="my-4 text-sm">Drag or paste videos here</div>
        </Viewport.Content>
      ) : (
        <Viewport.Content className="flex h-full w-full overflow-hidden">
          <LayoutGroup>
            <motion.div
              className="h-full"
              animate={{ width: inspector.show ? `calc(100% - ${resizableInspector.width}px)` : '100%' }}
              transition={
                // 关闭 inspector 的时候动画也瞬间完成，但是 animate width 的设置得继续保留
                // 不然现在一行内容比较少没有铺满屏幕宽度的时候，items 会和弹簧一样反复伸缩
                resizableInspector.isResizing || !inspector.show
                  ? { type: 'spring', duration: 0 }
                  : { type: 'spring', stiffness: 500, damping: 50 }
              }
            >
              <ExplorerLayout className="h-full w-full overflow-scroll" />
            </motion.div>
            <AnimatePresence mode="popLayout">
              {inspector.show && (
                <motion.div
                  layout
                  initial={{ x: '100%' }}
                  animate={{ x: 0 }}
                  exit={{ x: '100%' }}
                  transition={{ x: { type: 'spring', stiffness: 500, damping: 50 } }}
                  style={{ width: resizableInspector.width }}
                  className="flex h-full flex-none"
                >
                  <InspectorPane data={inspectorItemData} ref={resizableInspector.handleRef} />
                </motion.div>
              )}
            </AnimatePresence>
          </LayoutGroup>
        </Viewport.Content>
      )}
      <Footer /* Viewport.StatusBar */ />
      <AudioDialog />
    </Viewport.Page>
  )
}

export default function ExplorerPage() {
  const currentLibrary = useCurrentLibrary()
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

  // initialize explorer
  const materializedPath = useMemo(() => dirInSearchParams, [dirInSearchParams])
  // materializedPath 必须以 / 结尾, 调用 setMaterializedPath 的地方自行确保格式正确
  // const [materializedPath, setMaterializedPath] = useState<string>(dirInSearchParams)
  const [items, setItems] = useState<
    (ExtractExplorerItem<'FilePathDir'> | ExtractExplorerItem<'FilePathWithAssetObject'>)[] | null
  >(null)
  const explorer = useExplorerValue({
    items,
    materializedPath,
    settings: currentLibrary.librarySettings.explorer, // 渲染页面的时候 librarySettings 已经加载完毕
  })
  // 监听 explorer.settings 并保存到 librarySettings
  useEffect(() => {
    const librarySettings = currentLibrary.librarySettings
    if (
      explorer.settings.inspectorShow !== librarySettings.explorer.inspectorShow ||
      explorer.settings.inspectorSize !== librarySettings.explorer.inspectorSize ||
      explorer.settings.layout !== librarySettings.explorer.layout
    ) {
      currentLibrary.updateLibrarySettings({
        explorer: {
          inspectorShow: explorer.settings.inspectorShow,
          inspectorSize: explorer.settings.inspectorSize,
          layout: explorer.settings.layout,
        },
      })
    }
  }, [currentLibrary, explorer.settings])

  // fetch assets and update explorer items
  const assetsQueryParams = {
    materializedPath,
    includeSubDirs: explorer.settings.layout === 'media' ? true : false,
  }
  const assetsQuery = rspc.useQuery(['assets.list', assetsQueryParams], {
    // refetchOnWindowFocus: true,
    throwOnError: (e: RSPCError) => {
      console.log(e)
      return false // stop propagate throwing error
    },
  })
  const resetSelectedItems = explorer.resetSelectedItems
  useEffect(() => {
    if (assetsQuery.isSuccess) {
      /**
       * 在文件名中确保 10 > 2
       * TODO: 优化这部分代码，如果有分页，这个做法就失效了，后端要处理好。还有如果排序方式支持用户选，这里也要跟着改。
       */
      const sortedItems = assetsQuery.data.sort((a, b) =>
        a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' }),
      )
      const explorerItems = sortedItems.map((item) => {
        return item.isDir
          ? ({
              type: 'FilePathDir',
              filePath: item,
            } as ExtractExplorerItem<'FilePathDir'>)
          : ({
              type: 'FilePathWithAssetObject',
              filePath: item,
              assetObject: item.assetObject!,
            } as ExtractExplorerItem<'FilePathWithAssetObject'>)
      })
      setItems(explorerItems)
      const revealedItem = explorerItems.find((item) => item.filePath.id === initialRevealedFilePathId)
      if (revealedItem) {
        resetSelectedItems([revealedItem])
      } else {
        // resetSelectedItems()
        // 重新获取数据要清空选中的项目，以免出现不在列表中但是还被选中的情况
        // 实际上不需要，因为 selectedItems 是个 useMemo，已经过滤掉了 explorer.items 中不存在的 item
      }
    }
  }, [assetsQuery.isLoading, assetsQuery.isSuccess, assetsQuery.data, resetSelectedItems, initialRevealedFilePathId])

  if (assetsQuery.isError) {
    return <Viewport.Page className="text-ink/50 flex items-center justify-center">Failed to load assets</Viewport.Page>
  }

  return (
    <ExplorerViewContextProvider
      value={{
        contextMenu: (data) =>
          // checking for data.type is not necessary ...
          data.type === 'FilePathDir' || data.type === 'FilePathWithAssetObject' ? <ItemContextMenuV2 /> : null,
      }}
    >
      <ExplorerContextProvider explorer={explorer}>
        <InspectorProvider
          initialShow={
            // explorer 页面上的 inspectorShow 被托管到了 settings 上持久化，这里是设置初始值
            // 另外，如果有 initialRevealedFilePathId, 临时开启一下 inspector
            explorer.settings.inspectorShow || initialRevealedFilePathId !== null
          }
        >
          <_ExplorerPage />
        </InspectorProvider>
      </ExplorerContextProvider>
    </ExplorerViewContextProvider>
  )
}
