'use client'
import Icon from '@muse/ui/icons'
import { client, rspc, queryClient } from '@/lib/rspc'
import { Muse_Logo } from '@muse/assets/svgs'
import { useSearchParams } from 'next/navigation'
import Image from 'next/image'
import { useCallback, useEffect, useState } from 'react'

export default function LibrariesSelect() {
  const searchParams = useSearchParams()
  const librarySetIdInSearchParams = searchParams.get('setlibrary')

  const librariesQuery = rspc.useQuery(['libraries.list'])
  const libraryCreateMut = rspc.useMutation('libraries.create')

  const createLibrary = useCallback(() => {
    libraryCreateMut.mutate('a test library', {
      onSuccess: () => queryClient.invalidateQueries({
        queryKey: ['libraries.list']
      })
    })
  }, [libraryCreateMut])

  const [pending, setPending] = useState(false)
  const { mutateAsync: setCurrentLibraryAsync } = rspc.useMutation('libraries.set_current_library')
  const { mutateAsync: triggerUnfinishedAsync } = rspc.useMutation('video.tasks.trigger_unfinished')
  // const currentLibrary = useCurrentLibrary()
  const setLibraryById = useCallback(
    async (libraryId: string) => {
      setPending(true)
      await setCurrentLibraryAsync(libraryId)
      await triggerUnfinishedAsync(libraryId)
      location.reload()
      // setPending(false) 页面已经刷新了, 所以不需要再设置 setPending(false)
    },
    [setCurrentLibraryAsync, triggerUnfinishedAsync],
  )

  useEffect(() => {
    if (librarySetIdInSearchParams) {
      setLibraryById(librarySetIdInSearchParams)
    }
  }, [librarySetIdInSearchParams, setLibraryById])

  return (
    <div className="flex h-screen w-screen flex-col items-center justify-center bg-app">
      <Image src={Muse_Logo} alt="Muse" className="mb-4 h-8 w-8"></Image>
      {!pending && librariesQuery.isSuccess ? (
        <div className="my-4 w-80 rounded-md border border-app-line bg-app-box p-1 shadow-sm">
          {librariesQuery.data.length === 0 ? (
            <div className="px-3 py-2 text-center text-xs text-ink/60">No library has been created yet, continue by clicking &quot;Create&quot; below.</div>
          ) : (
            <div className="px-3 py-2 text-center text-xs text-ink/60">Select Library</div>
          )}
          {librariesQuery.data.map((library, index: number) => {
            return (
              <div
                key={library.id}
                className="flex items-center justify-start rounded-md px-3 py-2 hover:bg-app-hover"
                onClick={() => setLibraryById(library.id)}
              >
                <Image src={Muse_Logo} alt="Muse" className="h-8 w-8"></Image>
                <div className="mx-2 w-64 truncate text-xs font-semibold">
                  {library.title ?? 'Untitled'} ({library.id})
                </div>
              </div>
            )
          })}
          <div className="rounded-md px-3 py-2 hover:bg-app-hover" onClick={() => createLibrary()}>
            <div className="text-center text-sm">Create a library</div>
          </div>
        </div>
      ) : (
        <div className="text-ink/50 text-sm my-8 text-center">
          <Icon.Loading className="w-8 h-8 animate-spin inline-block" />
          <div className="mt-8">Loading library</div>
        </div>
      )}
    </div>
  )
}
