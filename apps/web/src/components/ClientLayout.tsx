'use client'
import LibrariesSelect from '@/components/LibrariesSelect'
import { useToast } from '@/components/Toast/use-toast'
import { CurrentLibrary, type Library } from '@/lib/library'
import { client, rspc } from '@/lib/rspc'
import { RSPCError } from '@rspc/client'
import { QueryClient } from '@tanstack/react-query'
import { convertFileSrc } from '@tauri-apps/api/tauri'
import { useCallback, useEffect, useState } from 'react'

export default function ClientLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  const { toast } = useToast()
  const [pending, setPending] = useState(true)
  const [library, setLibrary] = useState<Library | null>(null)
  // const [homeDir, setHomeDir] = useState<string|null>(null);

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

  useEffect(() => {
    blockCmdQ()

    const p1 = client
      .query(['libraries.get_current_library'])
      .then((library: Library) => setLibrary(library))
      .catch((error) => {
        console.log('libraries.get_current_library error:', error)
        setLibrary(null)
      })
    // const p2 = client.query(["files.home_dir"]).then((homeDir) => {
    //   setHomeDir(homeDir);
    // }).catch(error => {
    //   console.log('files.home_dir error:', error);
    //   setHomeDir(null);
    // });
    // Promise.all([p1, p2]).then(() => setPending(false));
    p1.then(() => setPending(false))
  }, [setLibrary, setPending, blockCmdQ])

  const setCurrentLibraryContext = useCallback(
    async (library: Library) => {
      setLibrary(library)
      setPending(true)
      try {
        await client.mutation(['libraries.set_current_library', library.id])
        // setPending(false);
        // 最后 reload 一下，用新的 library 请求数据过程中，页面上还残留着上个 library 已请求的数据
        // 既然要 reload，就不设置 setPending(false) 了
        location.reload()
      } catch (err) {
        console.error('CurrentLibraryStorage.set() error:', err)
      }
    },
    [setLibrary],
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
        if (typeof timestampInSecond === 'undefined') {
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

  /**
   * 这个配置只对 useQuery 和 useMutation 有效, 对使用 client.query 和 client.mutation 调用的请求无效
   */
  const queryClient: QueryClient = new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
        refetchOnWindowFocus: false,
      },
      mutations: {
        onSuccess: () => queryClient.invalidateQueries(),
        onError: (error) => {
          console.error(error)
          if (error instanceof RSPCError) {
            toast({ title: `请求出错 ${error.code}`, description: error.message, variant: 'destructive' })
          } else {
            toast({ title: '未知错误', description: error.message, variant: 'destructive' })
          }
        },
      },
    },
  })

  return pending ? (
    <></>
  ) : (
    <CurrentLibrary.Provider
      value={{
        ...(library ? library : {}),
        set: setCurrentLibraryContext,
        getFileSrc,
        getThumbnailSrc,
      }}
    >
      <rspc.Provider client={client} queryClient={queryClient}>
        {library?.id ? <>{children}</> : <LibrariesSelect />}
      </rspc.Provider>
    </CurrentLibrary.Provider>
  )
}

// TODO 实际上这个方法在 `content_library` 里已经实现了
// 可以用 bindgen 之类的办法从 rust 侧直接引用过来
function getFileShardHex(assetObjectHash: string) {
  return assetObjectHash.slice(0, 3)
}
