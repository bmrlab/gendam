'use client'
// import { DndContext } from '@/Explorer/components/Draggable/DndContext'
import DragOverlay from '@/Explorer/components/Draggable/DragOverlay'
import GridView from '@/Explorer/components/LayoutView/GridView'
import ListView from '@/Explorer/components/LayoutView/ListView'
import MediaView from '@/Explorer/components/LayoutView/MediaView'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { queryClient, rspc } from '@/lib/rspc'
import { DragCancelEvent, DragEndEvent, DragStartEvent, useDndMonitor } from '@dnd-kit/core'
import { HTMLAttributes, useCallback } from 'react'
import Selecto from 'react-selecto'
import { match, P } from 'ts-pattern'
import { type ExplorerItem } from '../types'

export default function ExplorerLayout({
  renderLayout,
  ...props
}: {
  renderLayout?: () => JSX.Element
} & HTMLAttributes<HTMLDivElement>) {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const moveMut = rspc.useMutation(['assets.move_file_path'])

  const handleMoveRequest = useCallback(
    async (active: ExplorerItem, target: ExplorerItem) => {
      if (active.type !== 'FilePath' || (target.type !== 'LibraryRoot' && target.type !== 'FilePath')) {
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
          target:
            target.type === 'LibraryRoot'
              ? null
              : {
                  id: target.filePath.id,
                  materializedPath: target.filePath.materializedPath,
                  isDir: target.filePath.isDir,
                  name: target.filePath.name,
                },
        })
      } catch (error) {}
      queryClient.invalidateQueries({
        queryKey: ['assets.list', { materializedPath: explorer.materializedPath }],
      })
      queryClient.invalidateQueries({
        queryKey: [
          'assets.list',
          {
            materializedPath:
              target.type === 'LibraryRoot' ? '/' : target.filePath.materializedPath + target.filePath.name + '/',
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
        // ExplorerDroppable 已经确保了 target 是 FilePath & isDir 或者是 LibraryRoot
        for (let active of explorerStore.drag.items) {
          if (active.type === 'FilePath' && target.type === 'FilePath' && active.filePath.id === target.filePath.id) {
            // 这个应该不会出现，因为设置了 disabled
            console.log('cannot move to self')
            continue
          }
          // console.log('move item', active, 'to', target)
          handleMoveRequest(active, target)
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

  /**
   * 参考了 spacedrive/interface/app/$libraryId/Explorer/useExplorerDnd.tsx
   * <DndContext> 在比较外层，但是绑定事件是在 Explorer 信息被初始化了以后
   */
  useDndMonitor({
    onDragStart,
    onDragEnd,
    onDragCancel,
  })

  function renderLayoutFromSettings() {
    function filtered<K extends ExplorerItem['type'], T extends Extract<ExplorerItem, K>>(
      items: ExplorerItem[],
      types: ExplorerItem['type'][],
    ): T[] {
      return items.filter((item) => types.includes(item.type)) as T[]
    }

    return match([explorer.settings.layout, explorer.items])
      .with(['grid', P.nonNullable], ([_, items]) => {
        return <GridView items={filtered(items, ['FilePath', 'SearchResult'])} />
      })
      .with(['list', P.nonNullable], ([_, items]) => {
        return <ListView items={filtered(items, ['FilePath', 'SearchResult'])} />
      })
      .with(['media', P.nonNullable], ([_, items]) => {
        return <MediaView items={filtered(items, ['FilePath', 'SearchResult'])} />
      })
      .otherwise(() => null)
  }

  if (!explorer.items || explorer.items.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-sm text-neutral-400">No items</p>
      </div>
    )
  }

  return (
    <div data-selecto-container onClick={() => explorer.resetSelectedItems()} {...props}>
      {/* <DndContext onDragStart={onDragStart} onDragEnd={onDragEnd} onDragCancel={onDragCancel}> */}
      <>
        {renderLayout ? renderLayout() : renderLayoutFromSettings()}
        <DragOverlay />
      </>
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
