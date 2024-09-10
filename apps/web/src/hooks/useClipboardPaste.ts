import { useCallback, useEffect, useState } from 'react'
import clipboard from 'tauri-plugin-clipboard-api'

export const useClipboardPaste = () => {
  const [filesPasted, setFilesPasted] = useState<string[]>([])
  const handlePaste = useCallback(async (e: ClipboardEvent) => {
    const files: string[] = []
    try {
      const res = await clipboard.readFiles()
      res.forEach((item) => {
        // clipboard 里面的文件名会被 encode，需要解码一下
        files.push(window.decodeURIComponent(item))
      })
    } catch (e) {}
    // 只阻拦文件
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
