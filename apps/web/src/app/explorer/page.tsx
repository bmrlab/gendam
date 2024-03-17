'use client'
import Explorer from '@/Explorer'
import { ExplorerContextProvider } from '@/Explorer/hooks'
import { ExplorerViewContextProvider, useExplorer } from '@/Explorer/hooks'
import { useExplorerStore } from '@/Explorer/store'
import { FilePathQueryResult } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { ContextMenuContent, ContextMenuItem } from '@muse/ui/v1/context-menu'
import classNames from 'classnames'
import { useRouter, useSearchParams } from 'next/navigation'
import { forwardRef, useCallback, useMemo, useState } from 'react'
import Footer from './_components/Footer'
import Header from './_components/Header'
import TitleDialog from './_components/TitleDialog'

const ItemContextMenu = forwardRef<typeof ContextMenuContent>(function ItemContextMenu({ ...prpos }, forwardedRef) {
  return (
    <ContextMenuContent
      ref={forwardedRef as any}
      className="w-60 rounded-md border border-neutral-100 bg-white p-1 shadow-lg"
      {...prpos}
    >
      <ContextMenuItem className="flex cursor-default items-center justify-start rounded-md px-2 py-2 hover:bg-neutral-200/60">
        <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">打开</div>
      </ContextMenuItem>
      <ContextMenuItem className="flex cursor-default items-center justify-start rounded-md px-2 py-2 hover:bg-neutral-200/60">
        <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">预览</div>
      </ContextMenuItem>
      <ContextMenuItem className="flex cursor-default items-center justify-start rounded-md px-2 py-2 hover:bg-neutral-200/60">
        <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">重命名</div>
      </ContextMenuItem>
      <ContextMenuItem
        className={classNames(
          'flex cursor-default items-center justify-start rounded-md px-2 py-2',
          'text-red-600 hover:bg-red-500/90 hover:text-white',
        )}
      >
        <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">删除</div>
      </ContextMenuItem>
    </ContextMenuContent>
  )
})

export default function ExplorerPage() {
  const searchParams = useSearchParams()
  let dirInSearchParams = searchParams.get('dir') || '/'
  if (!/^\/([^/\\:*?"<>|]+\/)+$/.test(dirInSearchParams)) {
    dirInSearchParams = '/'
  }

  const explorerStore = useExplorerStore()
  const router = useRouter()
  // currentPath 必须以 / 结尾, 调用 setCurrentPath 的地方自行确保格式正确
  // const [parentPath, setParentPath] = useState<string>(dirInSearchParams)
  const parentPath = useMemo(() => dirInSearchParams, [dirInSearchParams])

  const processVideoMut = rspc.useMutation(['assets.process_video_asset'])
  const renameMut = rspc.useMutation(['assets.rename_file_path'])
  const deleteMut = rspc.useMutation(['assets.delete_file_path'])

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

  const [mousePosition, setMousePosition] = useState<{ x: number; y: number }>({ x: 0, y: 0 })
  const handleMouseMove = useCallback(
    (event: React.MouseEvent) => {
      setMousePosition({ x: event.clientX, y: event.clientY })
    },
    [setMousePosition],
  )

  const [panelOpen, setPanelOpen] = useState<{ x: number; y: number; asset: FilePathQueryResult } | null>(null)
  const handleRightClick = useCallback(
    (asset: FilePathQueryResult) => {
      setPanelOpen({
        x: mousePosition.x,
        y: mousePosition.y,
        asset,
      })
    },
    [mousePosition, setPanelOpen],
  )

  const [renameDialog, setRenameDialog] = useState<{ asset: FilePathQueryResult } | null>(null)

  const onConfirmTitleInput = useCallback(
    (newName: string) => {
      if (!newName || !renameDialog) {
        return
      }
      console.log(newName, renameDialog.asset)
      renameMut.mutate({
        id: renameDialog.asset.id,
        path: parentPath,
        isDir: renameDialog.asset.isDir,
        oldName: renameDialog.asset.name,
        newName: newName,
      })
      setRenameDialog(null)
    },
    [renameDialog, setRenameDialog, renameMut, parentPath],
  )

  const onCancelTitleInput = useCallback(() => {
    setRenameDialog(null)
  }, [setRenameDialog])

  const handleDelete = useCallback(
    (asset: FilePathQueryResult) => {
      deleteMut.mutate({
        path: parentPath,
        name: asset.name,
      })
    },
    [deleteMut, parentPath],
  )

  const Panel: React.FC = () => {
    if (!panelOpen) return null
    return (
      <div
        className="fixed w-60 rounded-md border border-neutral-100 bg-white p-1 shadow-lg"
        style={{ left: panelOpen.x, top: panelOpen.y }}
        onClick={(e) => e.stopPropagation()}
      >
        <div className="flex cursor-default items-center justify-start rounded-md px-2 py-2 hover:bg-neutral-200/60">
          <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">
            打开 ({panelOpen.asset.name})
          </div>
        </div>
        <div className="flex cursor-default items-center justify-start rounded-md px-2 py-2 hover:bg-neutral-200/60">
          <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">预览</div>
        </div>
        <div className="my-1 h-px bg-neutral-100"></div>
        <div
          className="flex cursor-default items-center justify-start rounded-md px-2 py-2 hover:bg-neutral-200/60"
          onClick={() => {
            setPanelOpen(null)
            setRenameDialog({ asset: panelOpen.asset })
          }}
        >
          <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">重命名</div>
        </div>
        <div className="my-1 h-px bg-neutral-100"></div>
        <div
          className={classNames(
            'flex cursor-default items-center justify-start rounded-md px-2 py-2',
            'text-red-600 hover:bg-red-500/90 hover:text-white',
          )}
          onClick={() => {
            setPanelOpen(null)
            handleDelete(panelOpen.asset)
          }}
        >
          <div className="mx-1 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">删除</div>
        </div>
      </div>
    )
  }

  return (
    <ExplorerViewContextProvider
      value={{
        contextMenu: <ItemContextMenu />,
      }}
    >
      <ExplorerContextProvider explorer={explorer}>
        <div
          className="flex h-full flex-col"
          onClick={() => {
            setPanelOpen(null)
            explorer.resetSelectedItems()
          }}
          onMouseMove={handleMouseMove}
        >
          <Header></Header>
          <div className="flex-1">
            <Explorer></Explorer>
          </div>
          <Footer></Footer>
          <Panel />
          {renameDialog && <TitleDialog onConfirm={onConfirmTitleInput} onCancel={onCancelTitleInput} />}
        </div>
      </ExplorerContextProvider>
    </ExplorerViewContextProvider>
  )
}
