'use client'
import FoldersTree from '@/Explorer/components/FoldersTree'
import { useUploadQueueStore } from '@/store/uploadQueue'
import UploadQueue from '@/components/UploadQueue'
import { LibrariesListResult } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { GenDAM_Logo } from '@muse/assets/images'
import Icon from '@muse/ui/icons'
import { Button } from '@muse/ui/v2/button'
import classNames from 'classnames'
import Image from 'next/image'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { toast } from 'sonner'

export default function Sidebar() {
  const currentLibrary = useCurrentLibrary()
  const librariesQuery = rspc.useQuery(['libraries.list'])
  const panelRef = useRef<HTMLDivElement>(null)
  const [selectPanelOpen, setSelectPanelOpen] = useState(false)

  const uploadQueueStore = useUploadQueueStore()
  const [uploadQueueOpen, setUploadQueueOpen] = useState(false)

  const pathname = usePathname()
  const { data: version } = rspc.useQuery(['version'])

  const selected = useMemo<LibrariesListResult | undefined>(() => {
    if (librariesQuery.isSuccess) {
      return librariesQuery.data.find((library) => library.id === currentLibrary.id)
    }
  }, [currentLibrary.id, librariesQuery.data, librariesQuery.isSuccess])

  const switchLibrary = useCallback(
    async (library: LibrariesListResult) => {
      try {
        await currentLibrary.set(library.id)
      } catch (error) {
        toast.error('Failed to quit current library', {
          description: `${error}`,
        })
      }
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
    <div className="color-ink bg-sidebar relative flex h-screen w-60 flex-col items-stretch justify-start p-3">
      <section className="relative mb-6 mt-4">
        <div className="flex cursor-default items-center px-2" onClick={() => setSelectPanelOpen(true)}>
          <Image src={GenDAM_Logo} alt="Muse" className="h-8 w-8"></Image>
          <div className="mx-2 flex-1 overflow-hidden">
            <div className="truncate text-xs font-semibold">{selected?.title ?? 'Untitled'}</div>
          </div>
          <Icon.UpAndDownArrow className="h-4 w-4"></Icon.UpAndDownArrow>
        </div>
        {selectPanelOpen && librariesQuery.isSuccess ? (
          <div
            ref={panelRef}
            className="border-app-line bg-app-box text-ink absolute left-32 top-3
              z-10 w-72 rounded-md border p-1 shadow-sm"
          >
            {librariesQuery.data.map((library, index: number) => {
              return (
                <div
                  key={library.id}
                  className="hover:bg-app-hover/50 flex cursor-default items-center justify-start gap-2 rounded-md px-3 py-2"
                  onClick={() => switchLibrary(library)}
                >
                  <Image src={GenDAM_Logo} alt="Muse" className="h-9 w-9"></Image>
                  <div className="flex-1 overflow-hidden">
                    <div className="truncate text-xs font-semibold">{library.title}</div>
                    <div className="text-ink/50 truncate text-[0.6rem]">{library.id}</div>
                  </div>
                </div>
              )
            })}
          </div>
        ) : null}
      </section>

      <section className="text-sm">
        <Link href="/explorer" className={menuClassNames('/explorer')}>
          <Icon.File className="text-ink/70 mr-2 h-4 w-4" />
          <span>Library</span>
        </Link>
        <Link href="/search" className={menuClassNames('/search')}>
          <Icon.MagnifyingGlass className="text-ink/70 mr-2 h-4 w-4" />
          <span>Search</span>
        </Link>
        <Link href="/video-tasks" className={menuClassNames('/video-tasks')}>
          <Icon.Briefcase className="text-ink/70 mr-2 h-4 w-4" />
          <span>All jobs</span>
        </Link>
        {/* <Link href="/debug/ui" className={menuClassNames('/debug/ui')}>
          <span className="font-light text-neutral-400">Debug</span>
        </Link> */}
      </section>

      <FoldersTree className="-mx-3 my-4 flex-1" />

      <section>
        <div className="relative mb-2 flex items-center justify-start gap-1 text-sm">
          <Link href="/settings" className="block">
            <Button variant="ghost" size="sm" className="hover:bg-sidebar-hover h-7 w-7 p-1 transition-none">
              <Icon.Gear className="h-full w-full" />
            </Button>
          </Link>
          <Button
            variant="ghost"
            size="sm"
            className="hover:bg-sidebar-hover h-7 w-7 p-1 transition-none"
            onClick={() => {
              const theme = currentLibrary.librarySettings.appearanceTheme === 'dark' ? 'light' : 'dark'
              currentLibrary.updateLibrarySettings({
                appearanceTheme: theme,
              })
            }}
          >
            <Icon.Sun className="block h-full w-full dark:hidden" />
            <Icon.Moon className="hidden h-full w-full dark:block" />
          </Button>
          <div className="relative">
            <Button
              variant="ghost" size="sm" onClick={() => setUploadQueueOpen(!uploadQueueOpen)}
              className="hover:bg-sidebar-hover h-7 w-7 p-1 transition-none"
            >
              {uploadQueueStore.uploading || uploadQueueStore.queue.length ? (
                <div className="border-2 border-orange-400 p-[2px] h-full w-full rounded-full
                  animate-[flashstroke] duration-1000 repeat-infinite"></div>
              ) : (
                <div className="border border-current p-[2px] h-full w-full rounded-full scale-90">
                  <Icon.Check className="h-full w-full" />
                </div>
              )}
            </Button>
          </div>
          {uploadQueueOpen ? <UploadQueue close={() => setUploadQueueOpen(false)} /> : null}
        </div>
        <div className="px-1 text-xs text-neutral-400">v{version}</div>
      </section>
    </div>
  )
}
