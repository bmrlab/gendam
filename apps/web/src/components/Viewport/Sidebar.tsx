'use client'
import { useCurrentLibrary, type Library } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { Muse_Logo } from '@muse/assets/svgs'
import classNames from 'classnames'
import Image from 'next/image'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useCallback, useEffect, useRef, useState } from 'react'
import Icon from '@/components/Icon'

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
      await currentLibrary.set(library)
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
      'mb-1 block cursor-default rounded-md px-4 py-2 hover:bg-sidebar-hover',
      pathname === path && 'bg-sidebar-hover',
    )
  }

  return (
    <div className="h-screen w-60 color-ink bg-sidebar p-3 relative">
      <div className="relative my-4">
        <div className="flex cursor-default items-center justify-start" onClick={() => setSelectPanelOpen(true)}>
          <Image src={Muse_Logo} alt="Muse" className="h-8 w-8"></Image>
          <div className="mx-2 w-32 truncate text-xs font-semibold">
            {currentLibrary.settings?.title ?? "Untitled"} ({currentLibrary.id})
          </div>
          <Icon.arrowUpAndDown className="h-4 w-4"></Icon.arrowUpAndDown>
        </div>
        {selectPanelOpen && (
          <div
            ref={panelRef}
            className="absolute left-32 top-3 z-10 w-60 rounded-md
              border border-sidebar-line bg-sidebar-box p-1 shadow-sm"
          >
            {libraries.map((library, index: number) => {
              return (
                <div
                  key={library.id}
                  className="flex cursor-default items-center justify-start rounded-md px-3 py-2 hover:bg-sidebar-hover"
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
      <div className='absolute bottom-3 lett-3 text-sm'>
        <button
          onClick={() => {
            document.documentElement.classList.toggle('dark')
          }}
        >dark/light test</button>
      </div>
    </div>
  )
}
