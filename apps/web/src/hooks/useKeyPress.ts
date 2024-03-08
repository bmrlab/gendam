import { useCallback, useEffect, useState } from 'react'

export enum KeyType {
  Shift,
  Meta,
}

export default function useKeyPress(keyType: KeyType) {
  const [isPressed, setIsPressed] = useState(false)

  const isKeyDown = useCallback(
    (event: KeyboardEvent): boolean => {
      let isKey = false

      switch (keyType) {
        case KeyType.Shift:
          isKey = event.shiftKey
          break
        case KeyType.Meta:
          isKey = event.metaKey || event.ctrlKey
          break
      }
      return isKey
    },
    [keyType],
  )

  const isKeyUp = useCallback(
    (event: KeyboardEvent): boolean => {
      let isKey = false

      switch (keyType) {
        case KeyType.Shift:
          isKey = event.key === 'Shift'
          break
        case KeyType.Meta:
          isKey = event.key === 'Meta' || event.key === 'Control'
          break
      }
      return isKey
    },
    [keyType],
  )

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (isKeyDown(event)) {
        setIsPressed(true)
      }
    }

    const handleKeyUp = (event: KeyboardEvent) => {
      if (isKeyUp(event)) {
        setIsPressed(false)
      }
    }

    // 添加事件监听器
    document.addEventListener('keydown', handleKeyDown)
    document.addEventListener('keyup', handleKeyUp)

    // 清理事件监听器
    return () => {
      document.removeEventListener('keydown', handleKeyDown)
      document.removeEventListener('keyup', handleKeyUp)
    }
  }, [isKeyDown, isKeyUp])

  return isPressed
}
