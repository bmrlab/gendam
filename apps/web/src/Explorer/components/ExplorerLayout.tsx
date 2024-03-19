'use client'
import { DndContext } from '@/Explorer/components/DndContext'
import GridView from '@/Explorer/components/View/GridView'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { rspc } from '@/lib/rspc'
import { DragEndEvent, DragOverlay, DragStartEvent, DragCancelEvent } from '@dnd-kit/core'
import { Document_Light, Folder_Light } from '@muse/assets/images'
import Image from 'next/image'
import { useCallback } from 'react'
import { ExplorerItem } from '../types'

export default function Explorer() {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const moveMut = rspc.useMutation(['assets.move_file_path'])

  const handleDragStart = useCallback(
    (e: DragStartEvent) => {
      // console.log('onDragStart', e)
      const active = (e.active?.data?.current as ExplorerItem) ?? null
      if (active) {
        explorerStore.setDrag({ type: 'dragging', items: [active] })
      }
    },
    [explorerStore],
  )

  const handleDragEnd = useCallback(
    (e: DragEndEvent) => {
      // console.log('onDragEnd', e)
      const over = (e.over?.data?.current as ExplorerItem) ?? null
      const active = explorerStore.drag?.type === 'dragging' ? explorerStore.drag.items[0] : null
      if (over && active) {
        if (over.id === active.id) {
          // 这个应该不会出现，因为设置了 disabled
          console.log('cannot move to self')
          return
        }
        console.log('move item', active, 'to', over)
        moveMut.mutate({
          active: {
            id: active.id,
            path: active.materializedPath,
            isDir: active.isDir,
            name: active.name,
          },
          target: {
            id: over.id,
            path: over.materializedPath,
            isDir: over.isDir,
            name: over.name,
          },
        })
      }
      explorerStore.setDrag(null)
    },
    [explorerStore, moveMut],
  )

  const handleDragCancel = useCallback(
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
    <DndContext
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      onDragCancel={handleDragCancel}
    >
      <GridView items={explorer.items}></GridView>
      {/* <ListView items={explorer.items}></ListView> */}

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
    </DndContext>
  )
}
