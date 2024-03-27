'use client'
import { DndContext } from '@/Explorer/components/Draggable/DndContext'
import GridView from '@/Explorer/components/LayoutView/GridView'
import ListView from '@/Explorer/components/LayoutView/ListView'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { rspc } from '@/lib/rspc'
import { DragEndEvent, DragOverlay, DragStartEvent, DragCancelEvent } from '@dnd-kit/core'
import { Document_Light, Folder_Light } from '@muse/assets/images'
import Image from 'next/image'
import { useCallback, useState } from 'react'
import { ExplorerItem } from '../types'
import { FoldersDialog } from './FoldersDialog'

export default function Explorer() {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const moveMut = rspc.useMutation(['assets.move_file_path'])

  const handleMoveRequest = useCallback((active: ExplorerItem, over: ExplorerItem|null) => {
    moveMut.mutate({
      active: {
        id: active.id,
        path: active.materializedPath,
        isDir: active.isDir,
        name: active.name,
      },
      target: over ? {
        id: over.id,
        path: over.materializedPath,
        isDir: over.isDir,
        name: over.name,
      } : null,
    })
  }, [moveMut])

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

  const onTargetPathSelected = useCallback((target: ExplorerItem|null) => {
    for (let active of Array.from(explorer.selectedItems)) {
      // target 可以为空，为空就是根目录，这时候不需要检查 target.id !== active.id，因为根目录本身不会被移动
      if (!target || target.id !== active.id) {
        handleMoveRequest(active, target)
      }
    }
  }, [explorer, handleMoveRequest])

  if (!explorer.items || explorer.items.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-sm text-neutral-400">当前文件夹为空</p>
      </div>
    )
  }

  return (
    <DndContext
      onDragStart={onDragStart}
      onDragEnd={onDragEnd}
      onDragCancel={onDragCancel}
    >
      {/* <GridView items={explorer.items}></GridView> */}
      {/* <ListView items={explorer.items}></ListView> */}

      {function renderLayout() {
        switch (explorer.settings.layout) {
          case 'grid':
            return <GridView items={explorer.items} />
          case 'list':
            return <ListView items={explorer.items} />
          default:
            return null
        }
      }()}

      {!explorerStore.drag ? null : (
        <DragOverlay>
          {explorerStore.drag.items.map(
            (data) => (
              <div key={data.id} className="mb-2 flex items-center justify-start">
                <div className="h-8 w-8">
                  {data.isDir ? (
                    <Image src={Folder_Light} alt="folder" priority></Image>
                  ) : (
                    <Image src={Document_Light} alt="document" priority></Image>
                  )}
                </div>
                <div className="ml-2 w-32 rounded-lg bg-blue-600 p-1 text-white">
                  <div className="overflow-hidden overflow-ellipsis whitespace-nowrap text-xs">{data.name}</div>
                </div>
              </div>
            )
          )}
        </DragOverlay>
      )}

      <FoldersDialog onConfirm={onTargetPathSelected} />

    </DndContext>
  )
}
