import { useExplorerStore } from '@/Explorer/store'
import { DragOverlay as DragOverlayPrimitive, Modifier, type ClientRect } from '@dnd-kit/core'
import { getEventCoordinates } from '@dnd-kit/utilities'
import { Document_Light, Folder_Light } from '@muse/assets/images'
import Image from 'next/image'
import { PropsWithChildren, useEffect, useRef } from 'react'

const useSnapToCursorModifier = () => {
  const explorerStore = useExplorerStore()

  const initialRect = useRef<ClientRect | null>(null)

  const modifier: Modifier = ({ activatorEvent, activeNodeRect, transform }) => {
    if (!activeNodeRect || !activatorEvent) return transform

    const activatorCoordinates = getEventCoordinates(activatorEvent)
    if (!activatorCoordinates) return transform

    const rect = initialRect.current ?? activeNodeRect

    if (!initialRect.current) initialRect.current = activeNodeRect

    // Default offset so during drag the cursor doesn't overlap the overlay
    // which can cause issues with mouse events on other elements
    const offset = 12

    const offsetX = activatorCoordinates.x - rect.left
    const offsetY = activatorCoordinates.y - rect.top

    return {
      ...transform,
      x: transform.x + offsetX + offset,
      y: transform.y + offsetY + offset,
    }
  }

  useEffect(() => {
    if (!explorerStore.drag) initialRect.current = null
  }, [explorerStore.drag])

  return modifier
}

export default function DragOverlay({ children }: PropsWithChildren) {
  const explorerStore = useExplorerStore()
  const modifier = useSnapToCursorModifier()

  return explorerStore.drag ? (
    <DragOverlayPrimitive
      modifiers={[modifier]}
    >
      {explorerStore.drag.items.map((data) => (
        <div key={data.id} className="mb-2 flex w-60 items-center justify-start">
          <div className="h-6 w-6">
            {data.isDir ? (
              <Image src={Folder_Light} alt="folder" priority></Image>
            ) : (
              <Image src={Document_Light} alt="document" priority></Image>
            )}
          </div>
          <div className="ml-2 flex-1 overflow-hidden flex justify-start">
            <div className="py-1 px-2 truncate text-xs rounded-lg bg-blue-600 text-white">{data.name}</div>
          </div>
        </div>
      ))}
    </DragOverlayPrimitive>
  ) : (
    <></>
  )
}
