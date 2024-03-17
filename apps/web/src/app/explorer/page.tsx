'use client'
import ExplorerLayout from '@/Explorer/components/ExplorerLayout'
import { ExplorerContextProvider, ExplorerViewContextProvider, useExplorer } from '@/Explorer/hooks'
// import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
import { useSearchParams } from 'next/navigation'
import { useMemo } from 'react'
import Footer from './_components/Footer'
import Header from './_components/Header'
import ItemContextMenu from './_components/ItemContextMenu'

export default function ExplorerPage() {
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
        <div className="flex h-full flex-col"
          onClick={() => explorer.resetSelectedItems()}
          // onMouseMove={handleMouseMove}
        >
          <Header></Header>
          <div className="flex-1">
            <ExplorerLayout></ExplorerLayout>
          </div>
          <Footer></Footer>
        </div>
      </ExplorerContextProvider>
    </ExplorerViewContextProvider>
  )
}
