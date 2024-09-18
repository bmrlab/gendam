import { useCurrentLibrary } from '@/lib/library'
import { useCallback, useEffect, useRef, useState } from 'react'

export function useResizableInspector() {
  const currentLibrary = useCurrentLibrary()

  const ref = useRef<HTMLDivElement>(null)

  const [width, setWidth] = useState(currentLibrary.librarySettings.inspectorSize)

  const [isResizing, setIsResizing] = useState(false)
  const startXRef = useRef(0)
  const startWidthRef = useRef(width)

  const stopResizing = useCallback(() => {
    setIsResizing(false)
  }, [])

  const resize = useCallback(
    (e: MouseEvent) => {
      if (isResizing) {
        const deltaX = e.clientX - startXRef.current
        const newWidth = startWidthRef.current - deltaX
        // if (newWidth >= minWidth && newWidth <= maxWidth) {
        //   setWidth(newWidth);
        // }
        setWidth(newWidth)
      }
    },
    [isResizing],
  )

  useEffect(() => {
    const handleMouseUp = () => stopResizing()
    const handleMouseMove = (e: MouseEvent) => resize(e)

    if (isResizing) {
      window.addEventListener('mousemove', handleMouseMove)
      window.addEventListener('mouseup', handleMouseUp)
    }

    return () => {
      window.removeEventListener('mousemove', handleMouseMove)
      window.removeEventListener('mouseup', handleMouseUp)
    }
  }, [isResizing, resize, stopResizing])

  useEffect(() => {
    const handleMouseDown = (e: MouseEvent) => {
      setIsResizing(true)
      startXRef.current = e.clientX
      startWidthRef.current = width
    }

    const refCurrent = ref.current
    refCurrent?.addEventListener('mousedown', handleMouseDown)
    return () => {
      refCurrent?.removeEventListener('mousedown', handleMouseDown)
    }
  }, [width])

  const updateLibrarySettings = currentLibrary.updateLibrarySettings
  useEffect(() => {
    if (width && !isResizing) {
      updateLibrarySettings({ inspectorSize: width })
    }
  }, [width, isResizing, updateLibrarySettings])

  return { handleRef: ref, width, isResizing }
}
