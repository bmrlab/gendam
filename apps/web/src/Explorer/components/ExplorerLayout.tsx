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
import { ExplorerItem } from '../types'

export default function Explorer({ ...props }: HTMLAttributes<HTMLDivElement>) {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const moveMut = rspc.useMutation(['assets.move_file_path'])

  const handleMoveRequest = useCallback(
    async (active: ExplorerItem, target: ExplorerItem | null) => {
      try {
        await moveMut.mutateAsync({
          active: {
            id: active.id,
            materializedPath: active.materializedPath,
            isDir: active.isDir,
            name: active.name,
          },
          target: target
            ? {
                id: target.id,
                materializedPath: target.materializedPath,
                isDir: target.isDir,
                name: target.name,
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
            materializedPath: target ? target.materializedPath + target.name + '/' : '/',
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
          if (target.id !== active.id) {
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
          switch (explorer.settings.layout) {
            case 'grid':
              return <GridView items={explorer.items} />
            case 'list':
              return <ListView items={explorer.items} />
            case 'media':
              return <MediaView items={explorer.items} />
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
            const id = Number(el.getAttribute('data-selecto-item'))
            if (id) {
              explorer.addSelectedItemById(id)
              explorerStore.reset()
            }
          })
          e.removed.forEach((el) => {
            const id = Number(el.getAttribute('data-selecto-item'))
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
