'use client'
import GridView from '@/Explorer/components/View/GridView'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { DndContext, DragEndEvent, DragOverlay, DragStartEvent } from '@dnd-kit/core'
import { Document_Light, Folder_Light } from '@muse/assets/images'
import Image from 'next/image'
import { useCallback } from 'react'
import { ExplorerItem } from '../types'

export default function Explorer() {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()

  const handleDragStart = useCallback(
    async (e: DragStartEvent) => {
      console.log('onDragStart', e)
      const active = (e.active?.data?.current as ExplorerItem) ?? null
      if (active) {
        explorerStore.setDrag([active])
      }
    },
    [explorerStore],
  )

  const handleDragEnd = useCallback(
    async (e: DragEndEvent) => {
      console.log('onDragEnd', e)
      const over = (e.over?.data?.current as ExplorerItem) ?? null
      const active = explorerStore.drag?.items[0]
      if (over && active) {
        console.log("move item", active, "to", over)
      }
      explorerStore.setDrag([])
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

  // DndContext 放在 ExplorerLayout 是因为这里还会包括左侧文件夹树，也是 droppable 的
  return (
    <DndContext onDragStart={handleDragStart} onDragEnd={handleDragEnd}>
      <GridView items={explorer.items}></GridView>
      {/* <ListView items={explorer.items}></ListView> */}

      <DragOverlay>
        {explorerStore.isDraggingSelected
          ? explorerStore.drag?.items.map((data) => (
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
            ))
          : null}
      </DragOverlay>
    </DndContext>
  )
}
