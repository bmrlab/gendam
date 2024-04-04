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
  const { data: assets, refetch } =
    rspc.useQuery(['assets.list', { path: parentPath, dirsOnly: false }])

  useEffect(() => {
    // useUploadQueueStore.subscribe((e) => {})
    const uploading = uploadQueueStore.nextUploading()
    if (uploading) {
      uploadMut.mutate({
        path: uploading.path,
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
