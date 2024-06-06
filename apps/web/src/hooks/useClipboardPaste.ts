import { fiterFiles } from '@/lib/upload'
import { useEffect } from 'react'
import { toast } from 'sonner'
import clipboard from 'tauri-plugin-clipboard-api'
import { useUpload } from './useUpload'

export const useClipboardPaste = () => {
  const { handleSelectFiles } = useUpload()
  const handlePase = async (e: any) => {
    // 只阻拦文件
    const files: string[] = await clipboard.readFiles()
    if (files.length > 0) {
      e.preventDefault()
      const { supportedFiles, unsupportedExtensionsSet } = fiterFiles(files)
      if (supportedFiles.length > 0) {
        handleSelectFiles(supportedFiles)
      }
      if (Array.from(unsupportedExtensionsSet).length > 0) {
        toast.error(`Unsupported file types: ${Array.from(unsupportedExtensionsSet).join(',')}`)
      }
    }
  }

  useEffect(() => {
    window.addEventListener('paste', handlePase)
    return () => {
      window.removeEventListener('paste', handlePase)
    }
  }, [])
}
