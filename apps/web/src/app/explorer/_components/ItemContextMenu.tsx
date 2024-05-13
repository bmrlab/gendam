import { useExplorerContext } from '@/Explorer/hooks'
import { useExplorerStore } from '@/Explorer/store'
import { rspc, queryClient } from '@/lib/rspc'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
// import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { forwardRef, useCallback } from 'react'
// import { twx } from '@/lib/utils'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import { useMoveTargetSelected } from '@/hooks/useMoveTargetSelected'
import { useOpenFileSelection } from '@/hooks/useOpenFileSelection'
import { useInspector } from './Inspector'
import { ExplorerItem } from '@/Explorer/types'

type ItemContextMenuProps = {
  data: ExplorerItem
}

const ItemContextMenu = forwardRef<typeof ContextMenu.Content, ItemContextMenuProps>(function ItemContextMenuComponent(
  { data, ...prpos },
  forwardedRef,
) {
  const router = useRouter()

  // Explorer Component's State and Context
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  // Page Specific State and Context
  const inspector = useInspector()
  const { openFileSelection } = useOpenFileSelection()
  const { onMoveTargetSelected } = useMoveTargetSelected()

  // Shared State and Context
  const quickViewStore = useQuickViewStore()

  const deleteMut = rspc.useMutation(['assets.delete_file_path'])
  const metadataMut = rspc.useMutation(['assets.process_video_metadata'])
  const processJobsMut = rspc.useMutation(['video.tasks.regenerate'])
  const { data: stateData } = rspc.useQuery(['p2p.state'])
  const p2pMut = rspc.useMutation(['p2p.share'])

  /**
   * 这里都改成处理 selectedItems 而不只是处理当前的 item
   * ViewItem.tsx 里面的 handleContextMenuOpenChange 已经确保了当前 item 在 selectedItems 里
   */

  const handleOpen = useCallback(
    (e: Event) => {
      // e.stopPropagation()
      explorer.resetSelectedItems()
      explorerStore.reset()
      if (data.isDir) {
        let newPath = data.materializedPath + data.name + '/'
        router.push('/explorer?dir=' + newPath)
      } else if (data.assetObject) {
        const { name, assetObject } = data
        quickViewStore.open({ name, assetObject })
      }
    },
    [data, explorer, router, explorerStore, quickViewStore],
  )

  const handleShowInspector = useCallback(
    (e: Event) => {
      inspector.setShow(true)
    },
    [inspector],
  )

  const handleDelete = useCallback(
    async (e: Event) => {
      for (let item of Array.from(explorer.selectedItems)) {
        try {
          await deleteMut.mutateAsync({
            materializedPath: item.materializedPath,
            name: item.name,
          })
        } catch (error) {}
        queryClient.invalidateQueries({
          queryKey: ['assets.list', { materializedPath: item.materializedPath }],
        })
      }
      explorer.resetSelectedItems()
    },
    [deleteMut, explorer],
  )

  const handleRename = useCallback(
    (e: Event) => {
      explorerStore.setIsRenaming(true)
    },
    [explorerStore],
  )

  const handleProcessMetadata = useCallback(
    async (e: Event) => {
      for (let item of Array.from(explorer.selectedItems)) {
        if (!item.assetObject) {
          return
        }
        try {
          await metadataMut.mutateAsync(item.assetObject.id)
        } catch (error) {}
        queryClient.invalidateQueries({
          queryKey: ['assets.list', { materializedPath: item.materializedPath }],
        })
      }
    },
    [metadataMut, explorer],
  )

  const handleProcessJobs = useCallback(
    async (e: Event) => {
      for (let item of Array.from(explorer.selectedItems)) {
        if (!item.assetObject) {
          return
        }
        const assetObjectId = item.assetObject.id
        try {
          await processJobsMut.mutateAsync({ assetObjectId })
        } catch (error) {}
        queryClient.invalidateQueries({
          queryKey: ['tasks.list', { filter: { assetObjectId }}],
        })
      }
    },
    [processJobsMut, explorer],
  )

  const handleShare = useCallback(
    (peerId: string) => {
      let idList = Array.from(explorer.selectedItems).map((item) => {
        return item.id
      })
      p2pMut.mutate({ fileIdList: idList, peerId: peerId })
    },
    [explorer.selectedItems, p2pMut],
  )

  return (
    <ContextMenu.Content ref={forwardedRef as any} {...prpos} onClick={(e) => e.stopPropagation()}>
      <ContextMenu.Item onSelect={handleOpen} disabled={explorer.selectedItems.size > 1}>
        <div>Open</div>
      </ContextMenu.Item>
      <ContextMenu.Item onSelect={handleShowInspector} disabled={explorer.selectedItems.size > 1}>
        <div>Details</div>
      </ContextMenu.Item>
      {/* <ContextMenu.Item onSelect={() => {}} disabled={explorer.selectedItems.size > 1 }>
        <div>Quick view</div>
      </ContextMenu.Item> */}
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenu.Item
        onSelect={handleProcessMetadata}
        disabled={Array.from(explorer.selectedItems).some((item) => !item.assetObject)}
      >
        <div>Regen thumbnail</div>
      </ContextMenu.Item>
      <ContextMenu.Item
        onSelect={handleProcessJobs}
        disabled={Array.from(explorer.selectedItems).some((item) => !item.assetObject)}
      >
        <div>Re-process jobs</div>
      </ContextMenu.Item>
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenu.Item onSelect={() => openFileSelection().then((path) => onMoveTargetSelected(path))}>
        <div>Move</div>
      </ContextMenu.Item>
      <ContextMenu.Item onSelect={handleRename} disabled={explorer.selectedItems.size > 1}>
        <div>Rename</div>
      </ContextMenu.Item>
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenu.Sub>
        <ContextMenu.SubTrigger disabled={(stateData?.peers?.length ?? 0) === 0}>Share</ContextMenu.SubTrigger>
        <ContextMenu.SubContent>
          {stateData?.peers?.map((peer: { peer_id: string; metadata: { name?: string } }) => (
            <ContextMenu.Item key={peer.peer_id} onSelect={() => handleShare(peer.peer_id)}>
              {peer.metadata.name || peer.peer_id}
            </ContextMenu.Item>
          ))}
        </ContextMenu.SubContent>
      </ContextMenu.Sub>
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenu.Item variant="destructive" onSelect={handleDelete}>
        <div>Delete</div>
      </ContextMenu.Item>
    </ContextMenu.Content>
  )
})

export default ItemContextMenu
