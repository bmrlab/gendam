'use client'
import { CurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { Chevron_Double, Muse_Logo } from '@muse/assets/svgs'
import classNames from 'classnames'
import Image from 'next/image'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useCallback, useContext, useEffect, useRef, useState } from 'react'

export default function Sidebar() {
  const panelRef = useRef<HTMLDivElement>(null)
  const [selectPanelOpen, setSelectPanelOpen] = useState(false)
  const { data: libraries } = rspc.useQuery(['libraries.list'])
  const pathname = usePathname()
  const currentLibrary = useContext(CurrentLibrary)

  const switchLibrary = useCallback(
    async (libraryId: string) => {
      // console.log("switchLibrary");
      await currentLibrary.setContext(libraryId)
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

  return (
    <div className="min-h-full w-60 bg-neutral-100 p-3">
      <div className="relative my-4">
        <div className="flex cursor-default items-center justify-start" onClick={() => setSelectPanelOpen(true)}>
          <Image src={Muse_Logo} alt="Muse" className="h-8 w-8"></Image>
          <div className="mx-2 w-32 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs font-semibold">
            Muse ({currentLibrary.id})
          </div>
          <Image src={Chevron_Double} alt="Chevron_Double" className="h-4 w-4"></Image>
        </div>
        {selectPanelOpen && (
          <div
            ref={panelRef}
            className="absolute left-32 top-3 z-10 w-60 rounded-md
              border border-neutral-200 bg-neutral-100 p-1 shadow-sm"
          >
            {libraries?.map((libraryId: string, index: number) => {
              return (
                <div
                  key={libraryId}
                  className="flex cursor-default items-center justify-start rounded-md px-3
                    py-2 hover:bg-neutral-200"
                  onClick={() => switchLibrary(libraryId)}
                >
                  <Image src={Muse_Logo} alt="Muse" className="h-8 w-8"></Image>
                  <div className="mx-2 w-48 overflow-hidden overflow-ellipsis whitespace-nowrap text-xs font-semibold">
                    Muse ({libraryId})
                  </div>
                </div>
              )
            })}
          </div>
        )}
      </div>
      <div className="text-sm">
        <Link
          href="/library"
          className={classNames(
            'mb-1 block cursor-default rounded-md px-4 py-2 hover:bg-neutral-200',
            pathname === '/library' && 'bg-neutral-200',
          )}
        >
          本地文件(Test)
        </Link>
        <Link
          href="/assets"
          className={classNames(
            'mb-1 block cursor-default rounded-md px-4 py-2 hover:bg-neutral-200',
            pathname === '/assets' && 'bg-neutral-200',
          )}
        >
          素材库
        </Link>
        <Link
          href="/search"
          className={classNames(
            'mb-1 block cursor-default rounded-md px-4 py-2 hover:bg-neutral-200',
            pathname === '/search' && 'bg-neutral-200',
          )}
        >
          搜索
        </Link>
        <Link
          href="/video-tasks"
          className={classNames(
            'mb-1 block cursor-default rounded-md px-4 py-2 hover:bg-neutral-200',
            pathname === '/video-tasks' && 'bg-neutral-200',
          )}
        >
          视频任务
        </Link>
      </div>
    </div>
  )
}
