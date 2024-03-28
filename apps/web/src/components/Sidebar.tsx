'use client'
import { useCurrentLibrary, type Library } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { Chevron_Double, Muse_Logo } from '@muse/assets/svgs'
import classNames from 'classnames'
import Image from 'next/image'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useCallback, useEffect, useRef, useState } from 'react'

export default function Sidebar() {
  const librariesQuery = rspc.useQuery(['libraries.list'])
  const libraries = (librariesQuery.data ?? []) as Library[]

  const panelRef = useRef<HTMLDivElement>(null)
  const [selectPanelOpen, setSelectPanelOpen] = useState(false)
  const pathname = usePathname()
  const currentLibrary = useCurrentLibrary()

  const switchLibrary = useCallback(
    async (library: Library) => {
      // console.log("switchLibrary");
      await currentLibrary.setContext(library)
    },
    [currentLibrary],
  )

  useEffect(() => {
    function handleClickOutside(event: any) {
      // console.log(panelRef.current, event.target);
      if (panelRef.current && !panelRef.current.contains(event.target)) {
        setSelectPanelOpen(false)
      }
    }
    document.addEventListener('mousedown', handleClickOutside)
    return () => {
      document.removeEventListener('mousedown', handleClickOutside)
    }
  }, [])

  const menuClassNames = (path: string) => {
    return classNames(
      'mb-1 block cursor-default rounded-md px-4 py-2 hover:bg-neutral-200',
      pathname === path && 'bg-neutral-200',
    )
  }

  return (
    <div className="min-h-full w-60 bg-neutral-100 p-3">
      <div className="relative my-4">
        <div className="flex cursor-default items-center justify-start" onClick={() => setSelectPanelOpen(true)}>
          <Image src={Muse_Logo} alt="Muse" className="h-8 w-8"></Image>
          <div className="mx-2 w-32 truncate text-xs font-semibold">
            {currentLibrary.settings?.title ?? "Untitled"} ({currentLibrary.id})
          </div>
          <Image src={Chevron_Double} alt="Chevron_Double" className="h-4 w-4"></Image>
        </div>
        {selectPanelOpen && (
          <div
            ref={panelRef}
            className="absolute left-32 top-3 z-10 w-60 rounded-md
              border border-neutral-200 bg-neutral-100 p-1 shadow-sm"
          >
            {libraries.map((library, index: number) => {
              return (
                <div
                  key={library.id}
                  className="flex cursor-default items-center justify-start rounded-md px-3 py-2 hover:bg-neutral-200"
                  onClick={() => switchLibrary(library)}
                >
                  <Image src={Muse_Logo} alt="Muse" className="h-8 w-8"></Image>
                  <div className="mx-2 w-48 truncate text-xs font-semibold">
                    {library.settings?.title ?? 'Untitled'} ({library.id})
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </div>
      <div className="text-sm">
        <Link href="/explorer" className={menuClassNames('/explorer')}>
          素材库
        </Link>
        <Link href="/search" className={menuClassNames('/search')}>
          搜索
        </Link>
        <Link href="/video-tasks" className={menuClassNames('/video-tasks')}>
          视频任务
        </Link>
        <Link href="/debug/library" className={menuClassNames('/debug/library')}>
          <span className="font-light text-neutral-400">本地文件(Debug)</span>
        </Link>
      </div>
    </div>
  )
}
