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

  const {
    data: assets,
    isLoading,
    error,
    refetch,
  } = rspc.useQuery([
    'assets.list',
    {
      path: parentPath,
      dirsOnly: false,
    },
  ])

  const uploadFile = useCallback(async (file: FileItem) => {
    // uploadQueueStore.setUploading(true)
    // console.log("start upload", file.localFullPath)
    // await new Promise((resolve) => {
    //   setTimeout(() => resolve(null), 3000)
    // })
    await client.mutation([
      'assets.create_asset_object',
      {
        path: file.path,
        localFullPath: file.localFullPath,
      },
    ])
    refetch()
    // console.log("end upload", file.localFullPath)
  }, [refetch])

  useEffect(() => {
    // useUploadQueueStore.subscribe((e) => {})
    const uploading = uploadQueueStore.nextUploading()
    if (uploading) {
      uploadFile(uploading).then(() => {
        uploadQueueStore.completeUploading()
      }).catch((err) => {
        uploadQueueStore.failedUploading()
      })
    }
  }, [uploadQueueStore, uploadFile])

  const explorer = useExplorer({
    items: assets ?? null,
    parentPath: parentPath,
    settings: {
      layout: 'grid',
    },
  })

  // const [mousePosition, setMousePosition] = useState<{ x: number; y: number }>({ x: 0, y: 0 })
  // const handleMouseMove = useCallback(
  //   (event: React.MouseEvent) => {
  //     setMousePosition({ x: event.clientX, y: event.clientY })
  //   },
  //   [setMousePosition],
  // )

  const contextMenu = (data: ExplorerItem) => <ItemContextMenu data={data} />

  return (
    <ExplorerViewContextProvider value={{ contextMenu }}>
      <ExplorerContextProvider explorer={explorer}>

        <Viewport.Page
          onClick={() => explorer.resetSelectedItems()}
          // onMouseMove={handleMouseMove}
        >
          <Header></Header>
          <Viewport.Content>
            <ExplorerLayout></ExplorerLayout>
          </Viewport.Content>
          <Footer></Footer>

          <UploadQueue />
        </Viewport.Page>

      </ExplorerContextProvider>
    </ExplorerViewContextProvider>
  )
}
