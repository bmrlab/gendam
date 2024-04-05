'use client'
import ExplorerLayout from '@/Explorer/components/ExplorerLayout'
import { ExplorerContextProvider, ExplorerViewContextProvider, useExplorer } from '@/Explorer/hooks'
// import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import { client, rspc } from '@/lib/rspc'
import { FileItem, useUploadQueueStore } from '@/store/uploadQueue'
import { useSearchParams } from 'next/navigation'
import { useCallback, useEffect, useMemo } from 'react'
import Footer from './_components/Footer'
import Header from './_components/Header'
import ItemContextMenu from './_components/ItemContextMenu'
import UploadQueue from './_components/UploadQueue'
import Viewport from '@/components/Viewport'
import Inspector from './_components/Inspector'
import { RSPCError } from '@rspc/client'

export default function ExplorerPage() {
  const uploadQueueStore = useUploadQueueStore()

  const searchParams = useSearchParams()
  let dirInSearchParams = searchParams.get('dir') || '/'
  if (!/^\/([^/\\:*?"<>|]+\/)+$/.test(dirInSearchParams)) {
    dirInSearchParams = '/'
  }

  // const explorerStore = useExplorerStore()
  // currentPath 必须以 / 结尾, 调用 setCurrentPath 的地方自行确保格式正确
  // const [parentPath, setParentPath] = useState<string>(dirInSearchParams)
  const parentPath = useMemo(() => dirInSearchParams, [dirInSearchParams])

  const uploadMut = rspc.useMutation(['assets.create_asset_object'])
  const { data: assets, isError: assetsListFailed, refetch } =
    rspc.useQuery(['assets.list', { materializedPath: parentPath, dirsOnly: false }], {
      /**
       * 这样可以在删除/重命名/刷新metadata等操作执行以后自动刷新
       * 但现在看起来虽然全局设置了 refetchOnWindowFocus: false, 还是会自动刷新的
       */
      // refetchOnWindowFocus: true,
      throwOnError: (e: RSPCError) => {
        console.log(e)
        return false  // stop propagate throwing error
      },
    })

  useEffect(() => {
    // useUploadQueueStore.subscribe((e) => {})
    const uploading = uploadQueueStore.nextUploading()
    if (uploading) {
      uploadMut.mutate({
        materializedPath: uploading.path,
        localFullPath: uploading.localFullPath,
      }, {
        onSuccess: () => {
          uploadQueueStore.completeUploading()
          refetch()
        },
        onError: () => {
          uploadQueueStore.failedUploading()
        }
      })
    }
  }, [uploadQueueStore, uploadMut, refetch])

  const explorer = useExplorer({
    items: assets ?? null,
    parentPath: parentPath,
    settings: {
      layout: 'grid',
    },
  })

  const contextMenu = (data: ExplorerItem) => <ItemContextMenu data={data} />

  if (assetsListFailed) {
    return (
      <Viewport.Page className="flex items-center justify-center text-ink/50">
        Failed to load assets
      </Viewport.Page>
    )
  }

  return (
    <ExplorerViewContextProvider value={{ contextMenu }}>
      <ExplorerContextProvider explorer={explorer}>

        <Viewport.Page onClick={() => explorer.resetSelectedItems()}>
          <Header />

          <Viewport.Content className="flex items-start justify-start">
            <div className="flex-1 h-full">
              <ExplorerLayout></ExplorerLayout>
            </div>
            <Inspector />
          </Viewport.Content>

          <Footer />
          <UploadQueue />
        </Viewport.Page>

      </ExplorerContextProvider>
    </ExplorerViewContextProvider>
  )
}
