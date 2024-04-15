'use client'
import LibrariesSelect from '@/components/LibrariesSelect'
import Shared from '@/components/Shared'
import { LibrarySettings } from '@/lib/bindings'
import { CurrentLibrary, type Library } from '@/lib/library'
import { client, queryClient, rspc } from '@/lib/rspc'
import Icon from '@muse/ui/icons'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { useCallback, useEffect, useState } from 'react'
import { toast } from 'sonner'

export default function ClientLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  const [pending, setPending] = useState(true)
  const [library, setLibrary] = useState<Library | null>(null)
  const [librarySettings, setLibrarySettings] = useState<LibrarySettings | null>(null)

  const blockCmdQ = useCallback(() => {
    document.addEventListener('keydown', (event) => {
      if (event.metaKey && (event.key === 'q' || event.key === 'w')) {
        event.preventDefault()
        console.log('Cmd + Q is pressed.')
        alert('Cmd + Q 暂时禁用了，因为会导致 qdrant 不正常停止，TODO：实现 Cmd + Q 按了以后自行处理 app 退出')
        /**
         * https://github.com/bmrlab/tauri-dam-test-playground/issues/21#issuecomment-2002549684
         */
      }
    })
  }, [])

  const initLibraryData = useCallback(async () => {
    setPending(true)

    try {
      // get_current_library will load the library from store
      const library = await client.query(['libraries.get_current_library'])
      setLibrary(library)
      const librarySettings = await client.query(['libraries.get_library_settings'])
      setLibrarySettings(librarySettings)
      setPending(false)

      // 触发未完成的任务
      await client.mutation(['video.tasks.trigger_unfinished', library.id])
    } catch (error: any) {
      /**
       * 现在 libraries.get_current_library 有两种情况会报错
       * - 当前 library 为空
       * - 当前 library 正在 loading, 此时 error.message === "too many requests"
       *
       * 对于第二种情况，可以直接忽略请求，一般是因为 useEffect 二次执行造成的
       */
      if (error.message !== 'too many requests') {
        setPending(false)
        toast.error('Something went wrong getting library data, application will not start', {
          description: `${error}`,
        })
      }
    }
  }, [setLibrarySettings, setLibrary, setPending])

  useEffect(() => {
    // blockCmdQ()
    const disableContextMenu = (event: MouseEvent) => event.preventDefault()
    if (typeof window !== 'undefined') {
      window.addEventListener('contextmenu', disableContextMenu)
    }

    initLibraryData()

    return () => {
      if (typeof window !== 'undefined') {
        window.removeEventListener('contextmenu', disableContextMenu)
      }
    }
  }, [initLibraryData])

  useEffect(() => {
    const theme = librarySettings?.appearanceTheme
    if (theme === 'dark') {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [librarySettings?.appearanceTheme])

  const setCurrentLibrary = useCallback(
    async (libraryId: string) => {
      setLibrary(null)
      setPending(true)
      await client.mutation(['libraries.set_current_library', libraryId])
      initLibraryData()
    },
    [initLibraryData],
  )

  const updateLibrarySettings = useCallback(
    async (partialSettings: Partial<LibrarySettings>) => {
      if (!librarySettings) {
        return
      }
      const newSettings = { ...librarySettings, ...partialSettings }
      try {
        await client.mutation(['libraries.update_library_settings', newSettings])
        setLibrarySettings(newSettings)
      } catch (error) {
        toast.error('Failed to update library settings', {
          description: `${error}`,
        })
      }
    },
    [librarySettings],
  )

  const getFileSrc = useCallback(
    (assetObjectHash: string) => {
      if (!library) {
        return '/images/empty.png'
      }
      // const fileFullPath = library.dir + '/files/' + assetObjectHash
      const fileFullPath = `${library.dir}/files/${getFileShardHex(assetObjectHash)}/${assetObjectHash}`
      if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
        return convertFileSrc(fileFullPath)
      } else {
        return `http://localhost:3001/file/localhost/${fileFullPath}`
      }
    },
    [library],
  )

  const getThumbnailSrc = useCallback(
    (assetObjectHash: string, timestampInSecond?: number) => {
      // TODO remove the _timestampInSecond
      if (!library) {
        return '/images/empty.png'
      }

      const fileFullPath = (() => {
        if (typeof timestampInSecond === 'undefined' || timestampInSecond < 1) {
          return `${library.dir}/artifacts/${getFileShardHex(assetObjectHash)}/${assetObjectHash}/thumbnail.jpg`
        }

        return `${library.dir}/artifacts/${getFileShardHex(assetObjectHash)}/${assetObjectHash}/frames/${timestampInSecond}000.jpg`
      })()

      if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
        return convertFileSrc(fileFullPath)
      } else {
        return `http://localhost:3001/file/localhost/${fileFullPath}`
      }
    },
    [library],
  )

  return pending ? (
    <div className="text-ink/50 flex h-full w-full flex-col items-center justify-center">
      <Icon.Loading className="h-8 w-8 animate-spin" />
      <div className="mt-8 text-sm">Checking library data</div>
    </div>
  ) : !library || !librarySettings ? (
    <rspc.Provider client={client} queryClient={queryClient}>
      <LibrariesSelect setCurrentLibrary={setCurrentLibrary} />
    </rspc.Provider>
  ) : (
    <CurrentLibrary.Provider
      value={{
        id: library.id,
        dir: library.dir,
        librarySettings: librarySettings,
        updateLibrarySettings: updateLibrarySettings,
        set: setCurrentLibrary,
        getFileSrc,
        getThumbnailSrc,
      }}
    >
      <rspc.Provider client={client} queryClient={queryClient}>
        <>
          {children}
          <Shared />
        </>
      </rspc.Provider>
    </CurrentLibrary.Provider>
  )
}

// TODO 实际上这个方法在 `content_library` 里已经实现了
// 可以用 bindgen 之类的办法从 rust 侧直接引用过来
function getFileShardHex(assetObjectHash: string) {
  return assetObjectHash.slice(0, 3)
}
