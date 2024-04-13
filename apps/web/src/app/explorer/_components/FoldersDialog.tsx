import FileThumb from '@/Explorer/components/View/FileThumb'
import Icon from '@/components/Icon'
import { create } from 'zustand'
import { client, rspc } from '@/lib/rspc'
import { toast } from 'sonner'
import { Folder_Light } from '@muse/assets/images'
import { Button } from '@muse/ui/v2/button'
import { Dialog } from '@muse/ui/v2/dialog'
import { RSPCError } from '@rspc/client'
import classNames from 'classnames'
import Image from 'next/image'
import { useCallback, useEffect, useState } from 'react'
import { ExplorerItem } from '@/Explorer/types'

interface FoldersDialogState {
  open: boolean
  setOpen: (open: boolean) => void
}

export const useFoldersDialog = create<FoldersDialogState>((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
}))

export function FoldersDialog({ onConfirm }: { onConfirm: (path: ExplorerItem | null) => void }) {
  const foldersDialog = useFoldersDialog()
  const [currentPath, setCurrentPath] = useState<string>('/')

  const { data: dirs, isError: assetsListFailed } = rspc.useQuery(
    [
      'assets.list',
      {
        materializedPath: currentPath,
        isDir: true,
      },
    ],
    {
      enabled: foldersDialog.open,
      throwOnError: (error: RSPCError) => {
        console.log(error)
        return false // stop propagate throwing error
      },
    },
  )

  useEffect(() => {
    if (assetsListFailed) {
      toast.error(`Error get folders: ${currentPath}`, {
        duration: 5000,
      })
    }
  }, [assetsListFailed, currentPath])

  const [currentExplorerItem, setCurrentExplorerItem] = useState<ExplorerItem | null>(null)

  useEffect(() => {
    const match = currentPath.match(/^((\/[^/]+)*\/)([^/]+)\/$/)
    if (match) {
      const materializedPath = match[1]
      const name = match[3]
      client
        .query(['assets.get', { materializedPath, name }])
        .then((data) => {
          setCurrentExplorerItem(data)
        })
        .catch((error) => {
          toast.error(`Error fetch folder ${currentPath}`, {
            description: error.message
          })
        })
    } else {
      setCurrentExplorerItem(null)
    }
  }, [setCurrentExplorerItem, currentPath])

  const goto = useCallback(
    (data: ExplorerItem | '-1') => {
      if (data === '-1') {
        let newPath = currentPath.replace(/([^/]+)\/$/, '')
        setCurrentPath(newPath)
      } else {
        setCurrentPath(currentPath + data.name + '/')
      }
    },
    [currentPath, setCurrentPath],
  )

  return (
    <Dialog.Root
      open={foldersDialog.open}
      onOpenChange={(open) => foldersDialog.setOpen(open)}
    >
      <Dialog.Portal>
        <Dialog.Overlay onClick={(e) => e.stopPropagation()} />
        <Dialog.Content onClick={(e) => e.stopPropagation()} className="w-[40rem]">
          <div className="flex items-center justify-start border-b border-app-line px-4 py-3">
            <div className="text-sm">选择文件夹</div>
            <div className="ml-2 flex items-center gap-1 text-xs text-ink/50" onClick={() => goto('-1')}>
              <Icon.arrowUpLeft className="h-4 w-4" />
              <span>返回上一级</span>
            </div>
          </div>
          <div className="flex h-80 flex-wrap content-start items-start justify-start gap-4 overflow-auto p-4">
            {dirs && dirs.length > 0 ? (
              dirs.map((data) => (
                <div
                  key={data.id}
                  onDoubleClick={() => goto(data)}
                  className="flex cursor-default select-none flex-col items-center justify-start"
                >
                  <FileThumb data={data} className={classNames('mb-1 h-16 w-16 rounded-sm p-1')} />
                  <div className={classNames('w-16 rounded-sm px-1')}>
                    <div className="truncate text-center text-xs">{data.name}</div>
                  </div>
                </div>
              ))
            ) : (
              <div className="w-full py-8 text-center text-xs text-ink/50">当前文件夹为空</div>
            )}
          </div>
          <div className="flex items-center justify-end gap-2 border-t border-app-line px-4 py-2 text-sm">
            <div className="h-6 w-6">
              <Image src={Folder_Light} alt="folder" priority></Image>
            </div>
            <div className="text-xs">{currentPath}</div>
            <div className="mr-auto"></div>
            <Dialog.Close asChild>
              <Button variant="outline" size="sm">取消</Button>
            </Dialog.Close>
            <Dialog.Close asChild onClick={() => onConfirm(currentExplorerItem)}>
              <Button variant="accent" size="sm">选择当前文件夹</Button>
            </Dialog.Close>
            {/* {Array.from(explorer.selectedItems).map((item) => {
              return (
                <div key={item.id} className="text-xs text-neutral-400">
                  {item.name}
                </div>
              )
            })} */}
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
