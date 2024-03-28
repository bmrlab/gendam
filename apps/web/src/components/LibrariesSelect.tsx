'use client'
// import Link from "next/link";
import { useCurrentLibrary, type Library } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { Muse_Logo } from '@muse/assets/svgs'
import Image from 'next/image'
import { useCallback } from 'react'

export default function LibrariesSelect() {
  const librariesQuery = rspc.useQuery(['libraries.list'])
  const libraries = (librariesQuery.data ?? []) as Library[]

  const libraryMut = rspc.useMutation('libraries.create')

  const createLibrary = useCallback(() => {
    libraryMut.mutate('a test library')
  }, [libraryMut])

  const currentLibrary = useCurrentLibrary()
  const handleLibraryClick = useCallback(
    async (library: Library) => {
      await currentLibrary.setContext(library)
    },
    [currentLibrary],
  )

  return (
    <div className="flex h-screen w-screen flex-col items-center justify-center bg-white">
      <Image src={Muse_Logo} alt="Muse" className="mb-4 h-8 w-8"></Image>
      <div className="my-4 w-80 rounded-md border border-neutral-200 bg-neutral-100 p-1 shadow-sm">
        {libraries.length === 0 ? (
          <div className="px-3 py-2 text-center text-xs text-neutral-600">还未创建任何素材库，点击下方“创建”后继续</div>
        ) : (
          <div className="px-3 py-2 text-center text-xs text-neutral-600">选择素材库</div>
        )}
        {libraries.map((library, index: number) => {
          return (
            <div
              key={library.id}
              className="flex cursor-pointer items-center justify-start rounded-md px-3 py-2 hover:bg-neutral-200"
              onClick={() => handleLibraryClick(library)}
            >
              <Image src={Muse_Logo} alt="Muse" className="h-8 w-8"></Image>
              <div className="mx-2 w-64 truncate text-xs font-semibold">
                {library.settings?.title ?? 'Untitled'} ({library.id})
              </div>
            </div>
          )
        })}
        <div className="cursor-pointer rounded-md px-3 py-2 hover:bg-neutral-200" onClick={() => createLibrary()}>
          <div className="text-center text-sm">+ 创建素材库</div>
        </div>
      </div>
    </div>
  )
}
