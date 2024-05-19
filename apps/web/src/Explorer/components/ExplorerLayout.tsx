'use client'
import { DndContext } from '@/Explorer/components/Draggable/DndContext'
import DragOverlay from '@/Explorer/components/Draggable/DragOverlay'
import GridView from '@/Explorer/components/LayoutView/GridView'
import ListView from '@/Explorer/components/LayoutView/ListView'
import MediaView from '@/Explorer/components/LayoutView/MediaView'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { queryClient, rspc } from '@/lib/rspc'
import { DragCancelEvent, DragEndEvent, DragStartEvent } from '@dnd-kit/core'
import { HTMLAttributes, useCallback } from 'react'
import Selecto from 'react-selecto'
import { uniqueId, type ExplorerItem } from '../types'

type FilePathExplorerItem = Extract<ExplorerItem, { type: "FilePath" }>
type SearchResultExplorerItem = Extract<ExplorerItem, { type: "SearchResult" }>

export default function Explorer({ ...props }: HTMLAttributes<HTMLDivElement>) {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const moveMut = rspc.useMutation(['assets.move_file_path'])

  const handleMoveRequest = useCallback(
    async (active: ExplorerItem, target: ExplorerItem | null) => {
      if (active.type !== 'FilePath' || (target && target.type !== 'FilePath')) {
        // 现阶段只支持 FilePath 可以被移动
        return
      }
      try {
        await moveMut.mutateAsync({
          active: {
            id: active.filePath.id,
            materializedPath: active.filePath.materializedPath,
            isDir: active.filePath.isDir,
            name: active.filePath.name,
          },
          target: target
            ? {
                id: target.filePath.id,
                materializedPath: target.filePath.materializedPath,
                isDir: target.filePath.isDir,
                name: target.filePath.name,
              }
            : null,
        })
      } catch (error) {}
      queryClient.invalidateQueries({
        queryKey: ['assets.list', { materializedPath: explorer.materializedPath }],
      })
      queryClient.invalidateQueries({
        queryKey: [
          'assets.list',
          {
            materializedPath: target ? target.filePath.materializedPath + target.filePath.name + '/' : '/',
          },
        ],
      })
    },
    [explorer.materializedPath, moveMut],
  )

  const onDragStart = useCallback(
    (e: DragStartEvent) => {
      // console.log('onDragStart', e)
      const active = (e.active?.data?.current as ExplorerItem) ?? null
      if (active) {
        explorerStore.setIsRenaming(false)
        if (!explorer.isItemSelected(active)) {
          // 被 drag 的 item 没有被选中, 只处理这个 item
          explorer.resetSelectedItems([active])
          explorerStore.setDrag({ type: 'dragging', items: [active] })
        } else {
          // 被 drag 的 item 已经被选中, 处理所有被选中的 item
          const selectedItems = Array.from(explorer.selectedItems)
          explorerStore.setDrag({ type: 'dragging', items: selectedItems })
        }
      }
    },
    [explorerStore, explorer],
  )

  const onDragEnd = useCallback(
    (e: DragEndEvent) => {
      // console.log('onDragEnd', e)
      const target = (e.over?.data?.current as ExplorerItem) ?? null
      if (target && explorerStore.drag?.type === 'dragging') {
        for (let active of explorerStore.drag.items) {
          if (uniqueId(target) !== uniqueId(active)) {
            // console.log('move item', active, 'to', target)
            handleMoveRequest(active, target)
          } else {
            // 这个应该不会出现，因为设置了 disabled
            console.log('cannot move to self')
          }
        }
      }
      explorerStore.setDrag(null)
    },
    [explorerStore, handleMoveRequest],
  )

  const onDragCancel = useCallback(
    (e: DragCancelEvent) => {
      // console.log('onDragCancel', e)
      explorerStore.setDrag(null)
    },
    [explorerStore],
  )

  if (!explorer.items || explorer.items.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-sm text-neutral-400">当前文件夹为空</p>
      </div>
    )
  }

  return (
    <div
      data-selecto-container
      onClick={() => explorer.resetSelectedItems()}
      { ...props }
    >
      <DndContext onDragStart={onDragStart} onDragEnd={onDragEnd} onDragCancel={onDragCancel}>
        {/* <GridView items={explorer.items}></GridView> */}
        {/* <ListView items={explorer.items}></ListView> */}
        {(function renderLayout() {
          const items = explorer.items.filter(item => item.type === 'FilePath') as FilePathExplorerItem[]
          switch (explorer.settings.layout) {
            case 'grid':
              return <GridView items={items} />
            case 'list':
              return <ListView items={items} />
            case 'media':
              return <MediaView items={items} />
            default:
              return null
          }
        })()}
        <DragOverlay />
      </DndContext>
      <Selecto
        dragContainer="[data-selecto-container]"
        selectableTargets={['[data-selecto-item]']}
        onSelect={(e) => {
          e.added.forEach((el) => {
            const id = el.getAttribute('data-selecto-item')
            if (id) {
              explorer.addSelectedItemById(id)
              explorerStore.reset()
            }
          })
          e.removed.forEach((el) => {
            const id = el.getAttribute('data-selecto-item')
            if (id) {
              explorer.removeSelectedItemById(id)
              explorerStore.reset()
            }
          })
        }}
        hitRate={0}
        selectByClick={false}
        selectFromInside={false}
        preventClickEventOnDrag={true}
        continueSelect={false}
        continueSelectWithoutDeselect={true}
        ratio={0}
      />
    </div>
  )
}
