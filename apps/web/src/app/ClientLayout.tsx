'use client'
import DeviceAuth from '@/components/DeviceAuth'
import LibrariesSelect from '@/components/LibrariesSelect'
import Shared from '@/components/Shared'
import SonnerToaster from '@/components/SonnerToaster'
import Viewport from '@/components/Viewport'
import { DndContext } from '@/Explorer/components/Draggable/DndContext'
import { useP2PEvents } from '@/hooks/useP2PEvents'
import { Auth, LibrarySettings } from '@/lib/bindings'
import { AssetObjectType, AssetPreviewMetadata, CurrentLibrary, type Library } from '@/lib/library'
import { client, queryClient, rspc } from '@/lib/rspc'
import Icon from '@gendam/ui/icons'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { useCallback, useEffect, useState } from 'react'
import { toast } from 'sonner'
import { match } from 'ts-pattern'

const WebsocketLayout = () => {
  useP2PEvents()
  return <></>
}

const BlankPage = ({ children }: Readonly<{ children: React.ReactNode }>) => (
  <Viewport.Page>
    <Viewport.Toolbar className="h-8 border-none" /> {/* for window drag */}
    <Viewport.Content className="flex flex-col items-center justify-center">{children}</Viewport.Content>
  </Viewport.Page>
)

export default function ClientLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  const [pending, setPending] = useState(true)
  const [auth, setAuth] = useState<Auth | null>(null)
  const [library, setLibrary] = useState<Library | null>(null)
  const [librarySettings, setLibrarySettings] = useState<LibrarySettings | null>(null)

  useEffect(() => {
    const theme = librarySettings?.appearanceTheme
    if (theme === 'dark') {
      document.documentElement.classList.add('dark')
    } else {
      document.documentElement.classList.remove('dark')
    }
  }, [librarySettings?.appearanceTheme])

  const loadLibrary = useCallback(async (libraryId: string) => {
    try {
      const library = await client.mutation(['libraries.load_library', libraryId])
      setLibrary(library)
    } catch (error: any) {
      if (error?.code === 409 && error?.message === 'App is busy') {
        return { isBusy: true }
      } else {
        toast.error('Failed to load library', { description: error?.message || error })
        throw error
      }
    }
    try {
      const librarySettings = await client.query(['libraries.get_library_settings'])
      setLibrarySettings(librarySettings)
    } catch (error: any) {
      toast.error('Failed to get library settings', { description: error?.message || error })
      throw error
    }
    return {}
  }, [])

  const unloadLibrary = useCallback(async () => {
    try {
      await client.mutation(['libraries.unload_library'])
    } catch (error: any) {
      if (error?.code === 409 && error?.message === 'App is busy') {
        return { isBusy: true }
      } else {
        toast.error('Failed to unload library', { description: error?.message || error })
        throw error
      }
    }
    setLibrary(null)
    setLibrarySettings(null)
    return {}
  }, [])

  const listenToCmdQ = useCallback(() => {
    document.addEventListener('keydown', async (event) => {
      if (event.metaKey && (event.key === 'q' || event.key === 'w')) {
        event.preventDefault()
        toast.info('Cmd + Q is pressed, the app will be closed after library is unloaded.')
        // await new Promise((resolve) => setTimeout(resolve, 3000));
        await unloadLibrary()
        const { exit } = await import('@tauri-apps/api/process')
        exit(0)
        /**
         * console.log('Cmd + Q is pressed.')
         * alert('Cmd + Q 暂时禁用了，因为会导致 qdrant 不正常停止，TODO：实现 Cmd + Q 按了以后自行处理 app 退出')
         * https://github.com/bmrlab/tauri-dam-test-playground/issues/21#issuecomment-2002549684
         */
      }
    })
  }, [unloadLibrary])

  useEffect(() => {
    listenToCmdQ()
    const disableContextMenu = (event: MouseEvent) => event.preventDefault()
    if (typeof window !== 'undefined') {
      window.addEventListener('contextmenu', disableContextMenu)
    }

    setPending(true)
    Promise.all([client.query(['users.get']), client.query(['libraries.status'])])
      .then(async ([auth, { id, loaded, isBusy }]) => {
        // console.log(id, loaded, isBusy)
        setAuth(auth)
        if (!id) {
          setPending(false)
          return
        }
        if (isBusy) {
          toast.warning('App is busy, please try again later.')
          return
        }
        if (!loaded) {
          try {
            // app 刚启动的时候 loaded 是 false, 先 unload 一下以 kill qdrant
            // 现在 qdrant 没有了，服务端会在启动的时候首先 load_library，这里其实不会触发
            await unloadLibrary()
          } catch (error) {
            console.error(error)
          }
        }
        // loadLibrary 可以重复执行, 这里不需要判断 loaded 是否为 true
        // 虽然现在服务端会首先 loadLibrary, 这里还是要执行一下以获取完整的 library 信息
        // TODO：未来再简化这里的逻辑，可以考虑把 library 的信息放到 library.status 接口里面
        try {
          const { isBusy } = await loadLibrary(id)
          if (isBusy) {
            toast.info('App is busy', {
              description: 'The library is being loaded, please wait until it is done.',
            })
            return
          }
          setPending(false)
        } catch (error) {
          console.error(error)
          unloadLibrary()
            .then(() => setPending(false))
            .catch(console.error)
        }
      })
      .catch((error: any) => {
        console.error(error)
      })

    return () => {
      if (typeof window !== 'undefined') {
        window.removeEventListener('contextmenu', disableContextMenu)
      }
    }
  }, [loadLibrary, unloadLibrary, listenToCmdQ])

  const switchCurrentLibraryById = useCallback(
    async (libraryId: string) => {
      if (libraryId === library?.id) {
        return
      }
      setPending(true)
      try {
        // switch: unload then load
        await unloadLibrary()
        await loadLibrary(libraryId)
        setPending(false)
      } catch (error) {
        console.error(error)
      }
    },
    [loadLibrary, unloadLibrary, library?.id],
  )

  const updateLibrarySettings = useCallback(
    async (partialSettings: Partial<LibrarySettings>) => {
      if (!librarySettings) {
        return
      }
      const newSettings = { ...librarySettings, ...partialSettings }
      try {
        await client.mutation(['libraries.update_library_settings', newSettings])
        queryClient.invalidateQueries({ queryKey: ['libraries.list'] })
        setLibrarySettings(newSettings)
      } catch (error: any) {
        toast.error('Failed to update library settings', {
          description: error?.message || error,
        })
      }
    },
    [librarySettings],
  )

  const _constructStorageUri = useCallback((path: string) => {
    if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
      return convertFileSrc(path, 'storage')
    } else {
      return `http://localhost:3001/_storage/localhost/${path}`
    }
  }, [])

  const getFileSrc = useCallback(
    (assetObjectHash: string) => {
      if (!library) {
        return '/images/empty.png'
      }
      const path = `asset_object/${assetObjectHash}/file`
      return _constructStorageUri(path)
    },
    [library, _constructStorageUri],
  )

  const getThumbnailSrc = useCallback(
    (assetObjectHash: string, assetObjectType: AssetObjectType) => {
      if (!library) {
        return '/images/empty.png'
      }
      const prefix = `asset_object/${assetObjectHash}/artifacts`
      const fileFullPath = match(assetObjectType)
        .with('audio', () => `${prefix}/thumbnail.jpg`)
        .with('video', () => `${prefix}/thumbnail.jpg`)
        .with('image', () => `${prefix}/thumbnail.webp`)
        .with('webPage', () => `${prefix}/thumbnail.png`)
        .otherwise(() => `${prefix}/thumbnail.jpg`)
      return _constructStorageUri(fileFullPath)
    },
    [library, _constructStorageUri],
  )

  const getPreviewSrc: AssetPreviewMetadata = useCallback(
    (assetObjectHash, type, args1?: number) => {
      if (!library) {
        return '/images/empty.png'
      }
      const prefix = `asset_object/${assetObjectHash}/artifacts`
      return match(type)
        .with('audio', () => _constructStorageUri(`${prefix}/waveform.json`))
        .with('video', () => {
          const fileFullPath = (() => {
            if (typeof args1 === 'undefined' || args1 < 1) {
              return `${prefix}/thumbnail.jpg`
            }
            // see crates/content-base-task/src/video/frame.rs
            const frame_interval_seconds = 1
            args1 = Math.round(args1 / frame_interval_seconds) * frame_interval_seconds
            return `${prefix}/frames/${args1}000.jpg`
          })()

          return _constructStorageUri(fileFullPath)
        })
        .exhaustive()
    },
    [library, _constructStorageUri],
  )

  return (
    <rspc.Provider client={client} queryClient={queryClient}>
      <Viewport>
        {pending ? (
          <BlankPage>
            <Icon.Loading className="text-ink/50 h-8 w-8 animate-spin" />
            <div className="text-ink/50 mt-8 text-sm">Checking library data</div>
          </BlankPage>
        ) : !auth ? (
          <BlankPage>
            <DeviceAuth onSuccess={(auth) => setAuth(auth)} />
          </BlankPage>
        ) : !library || !librarySettings ? (
          <BlankPage>
            <LibrariesSelect switchCurrentLibraryById={switchCurrentLibraryById} />
          </BlankPage>
        ) : (
          <CurrentLibrary.Provider
            value={{
              id: library.id,
              dir: library.dir,
              librarySettings: librarySettings,
              updateLibrarySettings: updateLibrarySettings,
              switchCurrentLibraryById: switchCurrentLibraryById,
              getFileSrc,
              getThumbnailSrc,
              getPreviewSrc,
            }}
          >
            <DndContext>
              {/* see apps/web/src/Explorer/components/ExplorerLayout.tsx for more info about DndContext */}
              <Viewport.Sidebar />
              {children /* children should be a Viewport.Page element */}
              <Shared />
            </DndContext>
            <WebsocketLayout />
          </CurrentLibrary.Provider>
        )}
        <SonnerToaster />
      </Viewport>
    </rspc.Provider>
  )
}

// TODO 实际上这个方法在 `content_library` 里已经实现了
// 可以用 bindgen 之类的办法从 rust 侧直接引用过来
function getFileShardHex(assetObjectHash: string) {
  return assetObjectHash.slice(0, 3)
}
