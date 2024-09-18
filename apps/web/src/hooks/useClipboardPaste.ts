import { useCallback, useEffect, useState } from 'react'

export const useClipboardPaste = () => {
  const [filesPasted, setFilesPasted] = useState<File[]>([])
  const handlePaste = useCallback(async (e: ClipboardEvent) => {
    const files: File[] = []
    const items = e.clipboardData?.items
    if (items) {
      for (let i = 0; i < items.length; i++) {
        const item = items[i]
        if (item.kind === 'file') {
          const file = item.getAsFile()
          if (file) {
            files.push(file)
          }
        }
      }
    }
    if (files.length > 0) {
      e.preventDefault()
      setFilesPasted(files)
    }
  }, [])

  useEffect(() => {
    window.addEventListener('paste', handlePaste)
    return () => {
      window.removeEventListener('paste', handlePaste)
    }
  }, [handlePaste])

  // tip: 其他类型的 pasted object, 如果需要监听也在这里返回
  return { filesPasted }
}
