'use client'
import { useCurrentLibrary, type Library } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { Muse_Logo } from '@muse/assets/svgs'
import classNames from 'classnames'
import Image from 'next/image'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useCallback, useEffect, useRef, useState } from 'react'
import Icon from '@muse/ui/icons'
import { Button } from '@muse/ui/v2/button'

export default function Sidebar() {
  const librariesQuery = rspc.useQuery(['libraries.list'])
  const libraries = (librariesQuery.data ?? []) as Library[]

  const panelRef = useRef<HTMLDivElement>(null)
  const [selectPanelOpen, setSelectPanelOpen] = useState(false)
  const pathname = usePathname()
  const currentLibrary = useCurrentLibrary()
  const { data: version } = rspc.useQuery(['version'])

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
      'mb-1 block cursor-default rounded-md px-2 py-2 hover:bg-sidebar-hover flex items-center justify-start',
      pathname === path && 'bg-sidebar-hover',
    )
  }

  return (
    <div className="h-screen w-60 color-ink bg-sidebar p-3 relative">
      <div className="relative my-4">
        <div className="flex cursor-default items-center justify-start" onClick={() => setSelectPanelOpen(true)}>
          <Image src={Muse_Logo} alt="Muse" className="h-8 w-8"></Image>
          <div className="mx-2 w-32 truncate text-xs font-semibold">
            {currentLibrary.settings?.title ?? "Untitled"}
            {/* ({currentLibrary.id}) */}
          </div>
          <Icon.UpAndDownArrow className="h-4 w-4"></Icon.UpAndDownArrow>
        </div>
        {selectPanelOpen && (
          <div
            ref={panelRef}
            className="absolute left-32 top-3 z-10 w-72 rounded-md
              border border-app-line bg-app-box text-ink p-1 shadow-sm"
          >
            {libraries.map((library, index: number) => {
              return (
                <div
                  key={library.id}
                  className="flex gap-2 cursor-default items-center justify-start rounded-md px-3 py-2 hover:bg-app-hover/50"
                  onClick={() => switchLibrary(library)}
                >
                  <Image src={Muse_Logo} alt="Muse" className="h-9 w-9"></Image>
                  <div className="flex-1 overflow-hidden">
                    <div className="truncate text-xs font-semibold">{library.settings?.title ?? 'Untitled'}</div>
                    <div className="truncate text-[0.6rem] text-ink/50">{library.id}</div>
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </div>
      <div className="text-sm">
        <Link href="/explorer" className={menuClassNames('/explorer')}>
          <Icon.File className="h-4 w-4 text-ink/70 mr-2" />
          <span>素材库</span>
        </Link>
        <Link href="/search" className={menuClassNames('/search')}>
          <Icon.MagnifyingGlass className="h-4 w-4 text-ink/70 mr-2" />
          <span>搜索</span>
        </Link>
        <Link href="/video-tasks" className={menuClassNames('/video-tasks')}>
          <Icon.Briefcase className="h-4 w-4 text-ink/70 mr-2" />
          <span>视频任务</span>
        </Link>
        {/* <Link href="/debug/ui" className={menuClassNames('/debug/ui')}>
          <span className="font-light text-neutral-400">Debug</span>
        </Link> */}
      </div>
      <div className='absolute bottom-3 left-2'>
        <div className='text-sm flex items-center justify-start gap-1 mb-2'>
          <Link href="/settings" className='block'>
            <Button variant="ghost" size="sm" className="h-7 w-7 p-1">
              <Icon.Gear className="h-full w-full" />
            </Button>
          </Link>
          <Button
            variant="ghost" size="sm" className="h-7 w-7 p-1"
            onClick={() => document.documentElement.classList.toggle('dark')}
          >
            <span>Aa</span>
          </Button>
        </div>
        <div className='text-xs text-neutral-400 px-1'>v{version}</div>
      </div>
    </div>
  )
}
