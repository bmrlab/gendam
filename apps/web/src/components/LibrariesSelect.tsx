'use client'
// import Link from "next/link";
import { useCurrentLibrary, type Library } from '@/lib/library'
import { rspc, queryClient } from '@/lib/rspc'
import { Muse_Logo } from '@muse/assets/svgs'
import Image from 'next/image'
import { useCallback } from 'react'

export default function LibrariesSelect() {
  const librariesQuery = rspc.useQuery(['libraries.list'])
  const libraryMut = rspc.useMutation('libraries.create')

  const createLibrary = useCallback(() => {
    libraryMut.mutate('a test library', {
      onSuccess: () => queryClient.invalidateQueries({
        queryKey: ['libraries.list']
      })
    })
  }, [libraryMut])

  const currentLibrary = useCurrentLibrary()
  const handleLibraryClick = useCallback(
    async (library: Library) => {
      await currentLibrary.set(library)
    },
    [currentLibrary],
  )

  return (
    <div className="flex h-screen w-screen flex-col items-center justify-center bg-app">
      <Image src={Muse_Logo} alt="Muse" className="mb-4 h-8 w-8"></Image>
      {librariesQuery.isSuccess ? (
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
                onClick={() => handleLibraryClick(library)}
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
        <div className="text-ink/50 text-sm my-8">Loading ...</div>
      )}
    </div>
  )
}
