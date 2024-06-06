import { useExplorerContext } from '@/Explorer/hooks'
import { useExplorerStore } from '@/Explorer/store'
import { type ExplorerItem } from '@/Explorer/types'
import { useAudioDialog } from '@/components/Audio/AudioDialog'
import { useInspector } from '@/components/Inspector/store'
import { useQuickViewStore } from '@/components/Shared/QuickView/store'
import { useMoveTargetSelected } from '@/hooks/useMoveTargetSelected'
import { useOpenFileSelection } from '@/hooks/useOpenFileSelection'
import { type FilePath } from '@/lib/bindings'
import { queryClient, rspc } from '@/lib/rspc'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useRouter } from 'next/navigation'
import { forwardRef, useCallback, useMemo } from 'react'
import { toast } from 'sonner'

type ItemContextMenuProps = {
  data: FilePath
}

const ItemContextMenu = forwardRef<typeof ContextMenu.Content, ItemContextMenuProps>(function ItemContextMenuComponent(
  { data, ...prpos },
  forwardedRef,
) {
  const router = useRouter()

  // Explorer Component's State and Context
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const selectedFilePathItems = useMemo(() => {
    type T = Extract<ExplorerItem, { type: 'FilePath' }>
    const filtered = Array.from(explorer.selectedItems).filter((item) => item.type === 'FilePath') as T[]
    return filtered.map((item) => item.filePath)
  }, [explorer.selectedItems])

  // Page Specific State and Context
  const inspector = useInspector()
  const { openFileSelection } = useOpenFileSelection()
  const { onMoveTargetSelected } = useMoveTargetSelected()

  const audioDialog = useAudioDialog()

  // Shared State and Context
  const quickViewStore = useQuickViewStore()

  const deleteMut = rspc.useMutation(['assets.delete_file_path'])
  const metadataMut = rspc.useMutation(['assets.process_video_metadata'])
  const processJobsMut = rspc.useMutation(['video.tasks.regenerate'])
  const p2pStateQuery = rspc.useQuery(['p2p.state'])
  const p2pMut = rspc.useMutation(['p2p.share'])
  const { mutateAsync: uploadToS3 } = rspc.useMutation(['storage.upload_to_s3'])

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
      for (let item of selectedFilePathItems) {
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
    [selectedFilePathItems, deleteMut, explorer],
  )

  const handleRename = useCallback(
    (e: Event) => {
      explorerStore.setIsRenaming(true)
    },
    [explorerStore],
  )

  const handleProcessMetadata = useCallback(
    async (e: Event) => {
      for (let item of selectedFilePathItems) {
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
    [selectedFilePathItems, metadataMut],
  )

  const handleProcessJobs = useCallback(
    async (e: Event) => {
      for (let item of selectedFilePathItems) {
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
    [selectedFilePathItems, processJobsMut],
  )

  const handleShare = useCallback(
    (peerId: string) => {
      let idList = selectedFilePathItems.map((item) => {
        return item.id
      })
      p2pMut.mutate({ fileIdList: idList, peerId: peerId })
    },
    [selectedFilePathItems, p2pMut],
  )

  const handleUpload = useCallback(async () => {
    let hash = selectedFilePathItems.map((s) => s.assetObject?.hash).filter((s) => !!s) as string[]
    if (hash.length > 0) {
      try {
        await uploadToS3(hash)
        toast.success('Upload success')
      } catch (error) {
        toast.error('Upload failed')
      }
    }
  }, [selectedFilePathItems])

  return (
    <ContextMenu.Content
      ref={forwardedRef as any}
      {...prpos}
      onClick={(e) => e.stopPropagation()}
      className="data-[state=closed]:animate-none data-[state=closed]:duration-0"
    >
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
        disabled={selectedFilePathItems.some((item) => !item.assetObject)}
      >
        <div>Regen thumbnail</div>
      </ContextMenu.Item>
      <ContextMenu.Item
        onSelect={handleProcessJobs}
        disabled={selectedFilePathItems.some((item) => !item.assetObject)}
      >
        <div>Re-process jobs</div>
      </ContextMenu.Item>
      <ContextMenu.Item onSelect={() => {
        const items = selectedFilePathItems
        return items.length === 1 ? audioDialog.singleExport(items[0]) : audioDialog.batchExport(items)
      }}>
        <div>Export transcript</div>
      </ContextMenu.Item>
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenu.Item onSelect={() => openFileSelection().then((path) => onMoveTargetSelected(path))}>
        <div>Move</div>
      </ContextMenu.Item>
      <ContextMenu.Item onSelect={handleRename} disabled={explorer.selectedItems.size > 1}>
        <div>Rename</div>
      </ContextMenu.Item>
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenu.Item onSelect={handleUpload}>
        <div>Upload</div>
      </ContextMenu.Item>
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenu.Sub>
        <ContextMenu.SubTrigger disabled={(p2pStateQuery.data?.peers?.length ?? 0) === 0}>Share</ContextMenu.SubTrigger>
        <ContextMenu.SubContent>
          {p2pStateQuery.data?.peers?.map((peer: { peer_id: string; metadata: { name?: string } }) => (
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
