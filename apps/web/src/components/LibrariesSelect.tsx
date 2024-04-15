'use client'
import { queryClient, rspc } from '@/lib/rspc'
import { Muse_Logo } from '@muse/assets/svgs'
import Icon from '@muse/ui/icons'
import Image from 'next/image'
import { useCallback } from 'react'

export default function LibrariesSelect({
  setCurrentLibrary,
}: {
  setCurrentLibrary: (libraryId: string) => Promise<void>
}) {
  const librariesQuery = rspc.useQuery(['libraries.list'])
  const libraryCreateMut = rspc.useMutation('libraries.create')

  const createLibrary = useCallback(() => {
    libraryCreateMut.mutate('a test library', {
      onSuccess: () =>
        queryClient.invalidateQueries({
          queryKey: ['libraries.list'],
        }),
    })
  }, [libraryCreateMut])

  return (
    <div className="bg-app flex h-screen w-screen flex-col items-center justify-center">
      <Image src={Muse_Logo} alt="Muse" className="mb-4 h-8 w-8"></Image>
      {librariesQuery.isSuccess ? (
        <div className="border-app-line bg-app-box my-4 w-80 rounded-md border p-1 shadow-sm">
          {librariesQuery.data.length === 0 ? (
            <div className="text-ink/60 px-3 py-2 text-center text-xs">
              No library has been created yet, continue by clicking &quot;Create&quot; below.
            </div>
          ) : (
            <div className="text-ink/60 px-3 py-2 text-center text-xs">Select Library</div>
          )}
          {librariesQuery.data.map((library, index: number) => {
            return (
              <div
                key={library.id}
                className="hover:bg-app-hover flex items-center justify-start rounded-md px-3 py-2"
                onClick={() => setCurrentLibrary(library.id)}
              >
                <Image src={Muse_Logo} alt="Muse" className="h-8 w-8"></Image>
                <div className="mx-2 w-64 truncate text-xs font-semibold">
                  {library.title ?? 'Untitled'} ({library.id})
                </div>
              </div>
            )
          })}
          <div className="hover:bg-app-hover rounded-md px-3 py-2" onClick={() => createLibrary()}>
            <div className="text-center text-sm">Create a library</div>
          </div>
        </div>
      ) : (
        <div className="text-ink/50 my-8 text-center text-sm">
          <Icon.Loading className="inline-block h-8 w-8 animate-spin" />
          <div className="mt-8">Loading library</div>
        </div>
      )}
    </div>
  )
}
