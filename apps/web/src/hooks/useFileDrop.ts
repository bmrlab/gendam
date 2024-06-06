import { listen } from '@tauri-apps/api/event'
import { useEffect, useState } from 'react'

export const useFileDrop = () => {
  const [filesDropped, setFilesDropped] = useState<string[]>([])

  useEffect(() => {
    if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
      let unlisten: () => void
      let isExit = false
      listen('tauri://file-drop', (event) => {
        if (isExit) {
          return
        }
        const files = event.payload as string[]
        console.log('files dropped', files)
        setFilesDropped(files)
      }).then((_unlisten) => {
        unlisten = _unlisten
      })
      return () => {
        isExit = true
        if (unlisten) {
          unlisten()
        }
      }
    }
  }, [])

  return { filesDropped }
}
