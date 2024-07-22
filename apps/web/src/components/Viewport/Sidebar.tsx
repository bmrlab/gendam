'use client'
import FoldersTree from '@/Explorer/components/FoldersTree'
import { ExplorerApiContextProvider } from '@/Explorer/hooks/useExplorerApi'
import UploadQueue from '@/components/UploadQueue'
import { useUploadQueueStore } from '@/components/UploadQueue/store'
import { useUpdater } from '@/hooks/useUpdater'
import { LibrariesListResult } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { GenDAM_Logo } from '@gendam/assets/images'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import { Popover } from '@gendam/ui/v2/popover'
import classNames from 'classnames'
import Image from 'next/image'
import Link from 'next/link'
import { usePathname } from 'next/navigation'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { toast } from 'sonner'

const Version = () => {
  const { currentVersion, updateStatus, updateError } = useUpdater()
  // "PENDING" | "ERROR" | "DONE" | "UPTODATE"
  useEffect(() => {
    if (updateStatus === 'DONE') {
      toast.success('Update completed', {
        description: 'GenDAM has been updated to the latest version, it will apply after you restart the app.',
        duration: 86400 * 1000,
      })
    } else if (updateStatus === 'ERROR') {
      toast.error('Update failed', {
        description: updateError,
        duration: 30 * 1000,
      })
    }
  }, [updateError, updateStatus])

  return (
    <div className="text-ink/50 flex items-center justify-start gap-2 px-1 text-xs">
      <div>v{currentVersion}</div>
      {updateStatus === 'PENDING' ? (
        <>
          <Icon.Loading className="h-3 w-3 animate-spin" />
          <div className="text-ink/30">Updating</div>
        </>
      ) : null}
    </div>
  )
}

export default function Sidebar() {
  const pathname = usePathname()
  const currentLibrary = useCurrentLibrary()
  const librariesQuery = rspc.useQuery(['libraries.list'])

  const uploadQueueStore = useUploadQueueStore()
  const [uploadQueueOpen, setUploadQueueOpen] = useState(false)

  const { data: filePathsInProcess } = rspc.useQuery(['tasks.get_assets_in_process'], {
    refetchInterval: 5000,
  })
  const setInProcessItems = uploadQueueStore.setInProcessItems
  useEffect(() => {
    setInProcessItems(filePathsInProcess || [])
  }, [setInProcessItems, filePathsInProcess])

  const selectedLibrary = useMemo<LibrariesListResult | undefined>(() => {
    if (librariesQuery.isSuccess) {
      return librariesQuery.data.find((library) => library.id === currentLibrary.id)
    }
  }, [currentLibrary.id, librariesQuery.data, librariesQuery.isSuccess])

  const switchLibrary = useCallback(
    async (library: LibrariesListResult) => {
      try {
        await currentLibrary.switchCurrentLibraryById(library.id)
      } catch (error) {
        toast.error('Failed to quit current library', {
          description: `${error}`,
        })
      }
    },
    [currentLibrary],
  )

  const [lastQueueLength, setLastQueueLength] = useState(0)
  useEffect(() => {
    if (uploadQueueStore.queue.length > lastQueueLength) {
      setUploadQueueOpen(true)
    }
    setLastQueueLength(uploadQueueStore.queue.length)
  }, [lastQueueLength, uploadQueueStore.queue.length])

  const menuClassNames = (path: string) => {
    return classNames(
      'mb-1 block cursor-default rounded-md px-2 py-2 hover:bg-sidebar-hover gap-2 flex items-center justify-start',
      pathname === path && 'bg-sidebar-hover',
    )
  }

  return (
    <ExplorerApiContextProvider
      value={{
        listApi: 'assets.list',
        moveApi: 'assets.move_file_path',
      }}
    >
      <div className="text-ink bg-sidebar relative flex h-full w-60 flex-col items-stretch justify-start">
        <div data-tauri-drag-region className="h-10"></div>
        <section className="mx-3 mb-6 mt-2">
          <Popover.Root>
            <Popover.Trigger asChild disabled={!librariesQuery.isSuccess || !librariesQuery.data.length}>
              <div className="flex cursor-default items-center">
                <Image src={GenDAM_Logo} alt="GenDAM" className="h-8 w-8"></Image>
                <div className="mx-2 flex-1 overflow-hidden">
                  <div className="truncate text-xs font-semibold">{selectedLibrary?.title ?? 'Untitled'}</div>
                </div>
                <Icon.UpAndDownArrow className="h-4 w-4"></Icon.UpAndDownArrow>
              </div>
            </Popover.Trigger>
            <Popover.Portal>
              <Popover.Content side="right" align="start" sideOffset={0} alignOffset={0}>
                <div className="border-app-line bg-app-box text-ink w-72 rounded-md border p-1 shadow-sm">
                  {librariesQuery.data?.map((library, index: number) => {
                    return (
                      <div
                        key={library.id}
                        className="hover:bg-app-hover/50 flex cursor-default items-center justify-start gap-2 rounded-md px-3 py-2"
                        onClick={() => switchLibrary(library)}
                      >
                        <Image src={GenDAM_Logo} alt="GenDAM" className="h-9 w-9"></Image>
                        <div className="flex-1 overflow-hidden">
                          <div className="truncate text-xs font-semibold">{library.title}</div>
                          <div className="text-ink/50 truncate text-[0.6rem]">{library.id}</div>
                        </div>
                      </div>
                    )
                  })}
                </div>
              </Popover.Content>
            </Popover.Portal>
          </Popover.Root>
        </section>

        <section className="mx-3 text-sm">
          <Link href="/explorer" className={menuClassNames('/explorer')}>
            <Icon.File className="text-ink/70 h-4 w-4" />
            <span>Library</span>
          </Link>
          <Link href="/search" className={menuClassNames('/search')}>
            <Icon.MagnifyingGlass className="text-ink/70 h-4 w-4" />
            <span>Search</span>
          </Link>
          <Link href="/trash" className={menuClassNames('/trash')}>
            <Icon.Trash className="text-ink/70 h-4 w-4" />
            <span>Trash</span>
          </Link>
          {/* <Link href="/video-tasks" className={menuClassNames('/video-tasks')}>
          <Icon.Briefcase className="text-ink/70 h-4 w-4" />
          <span>All jobs</span>
          {inCompletedTasks?.data.length ? (
            <Icon.FlashStroke className="h-3 w-3 text-orange-400" />
          ) : null}
        </Link> */}
          {/* <Link href="/debug/ui" className={menuClassNames('/debug/ui')}>
          <span className="font-light text-neutral-400">Debug</span>
        </Link> */}
        </section>

        <FoldersTree className="my-4 flex-1" />

        <section className="relative mx-3 mb-2 flex items-center justify-start gap-1 text-sm">
          <Link href="/settings" className="block">
            <Button variant="ghost" size="sm" className="hover:bg-sidebar-hover h-7 w-7 p-1 transition-none">
              <Icon.Settings className="h-full w-full" />
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
          <Popover.Root open={uploadQueueOpen} onOpenChange={(open) => setUploadQueueOpen(open)}>
            <Popover.Trigger asChild>
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setUploadQueueOpen(!uploadQueueOpen)}
                className="hover:bg-sidebar-hover h-7 w-7 p-1 transition-none"
              >
                {uploadQueueStore.uploading || uploadQueueStore.queue.length || uploadQueueStore.inProcess.length ? (
                  <Icon.FlashStroke bold className="h-4 w-4 text-orange-400" />
                ) : (
                  <div className="h-full w-full scale-90 rounded-full border border-current p-[2px]">
                    <Icon.Check className="h-full w-full" />
                  </div>
                )}
              </Button>
            </Popover.Trigger>
            <Popover.Portal>
              <Popover.Content side="bottom" align="start" sideOffset={8}>
                <UploadQueue close={() => setUploadQueueOpen(false)} />
              </Popover.Content>
            </Popover.Portal>
          </Popover.Root>
        </section>

        <section className="mx-3 mb-3">
          <Version />
        </section>
      </div>
    </ExplorerApiContextProvider>
  )
}
