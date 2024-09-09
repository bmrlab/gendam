import { useExplorerContext } from '@/Explorer/hooks'
import { ExtractExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { useCallback, useMemo } from 'react'
import { BaseContextMenuItem } from './types'

function withShareExplorerItem(BaseComponent: BaseContextMenuItem) {
  return () => {
    const explorer = useExplorerContext()

    const p2pStateQuery = rspc.useQuery(['p2p.state'])
    const p2pMut = rspc.useMutation(['p2p.share'])

    type T = ExtractExplorerItem<'FilePath'>
    const selectedFilePathItems: T[] = useMemo(() => {
      return Array.from(explorer.selectedItems).filter((item) => item.type === 'FilePath') as T[]
    }, [explorer.selectedItems])

    const handleShare = useCallback(
      (peerId: string) => {
        let idList = selectedFilePathItems.map((item) => {
          return item.filePath.id
        })
        p2pMut.mutate({ fileIdList: idList, peerId: peerId })
      },
      [selectedFilePathItems, p2pMut],
    )

    return (
      <ContextMenu.Sub>
        <ContextMenu.SubTrigger disabled={(p2pStateQuery.data?.peers?.length ?? 0) === 0}>
          <BaseComponent />
        </ContextMenu.SubTrigger>
        <ContextMenu.SubContent>
          {p2pStateQuery.data?.peers?.map((peer: { peer_id: string; metadata: { name?: string } }) => (
            <ContextMenu.Item key={peer.peer_id} onSelect={() => handleShare(peer.peer_id)}>
              {peer.metadata.name || peer.peer_id}
            </ContextMenu.Item>
          ))}
        </ContextMenu.SubContent>
      </ContextMenu.Sub>
    )
  }
}

export default withShareExplorerItem(() => <div>Share</div>)
