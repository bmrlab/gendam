import FileThumb from '@/Explorer/components/View/FileThumb'
import { rspc, client } from '@/lib/rspc'
import { Folder_Light } from '@muse/assets/images'
import { Button } from '@muse/ui/v1/button'
import { DialogPrimitive as Dialog } from '@muse/ui/v1/dialog'
import Icon from '@/components/Icon'
import classNames from 'classnames'
import Image from 'next/image'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { useExplorerContext } from '../hooks'
import { useExplorerStore } from '../store'
import { ExplorerItem } from '../types'

export function FoldersDialog({ onConfirm }: { onConfirm: (path: ExplorerItem | null) => void }) {
  const explorer = useExplorerContext()
  const explorerStore = useExplorerStore()
  const [currentPath, setCurrentPath] = useState<string>('/')

  const { data: dirs } = rspc.useQuery(['assets.list', { path: currentPath, dirsOnly: true }])
  const [currentExplorerItem, setCurrentExplorerItem] = useState<ExplorerItem|null>(null)

  useEffect(() => {
    const match = currentPath.match(/^((\/[^/]+)*\/)([^/]+)\/$/)
    if (match) {
      const path = match[1]
      const name = match[3]
      client.query(["assets.get", { path, name }]).then((data) => {
        setCurrentExplorerItem(data)
      })
    } else {
      setCurrentExplorerItem(null)
    }
  }, [setCurrentExplorerItem, currentPath])

  const goto = useCallback(
    (data: ExplorerItem | '-1') => {
      if (data === '-1') {
        let newPath = currentPath.replace(/([^/]+)\/$/, "")
        setCurrentPath(newPath)
      } else {
        setCurrentPath(currentPath + data.name + '/')
      }
    },
    [currentPath, setCurrentPath]
  )

  return (
    <Dialog.Root
      open={explorerStore.isFoldersDialogOpen}
      onOpenChange={(open) => explorerStore.setIsFoldersDialogOpen(open)}
    >
      <Dialog.Portal>
        <Dialog.Overlay
          className="fixed inset-0 z-50 bg-black/30  data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
          onClick={(e) => e.stopPropagation()}
        />
        <Dialog.Content
          className={classNames(
            'fixed z-50 left-[50%] top-[50%] w-[38rem] max-w-full translate-x-[-50%] translate-y-[-50%] overflow-auto',
            'border border-app-line text-ink bg-app-box shadow-lg rounded-lg',
            'duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%]')}
          onClick={(e) => e.stopPropagation()}
        >
          <div className="flex items-center justify-start border-b border-app-line px-4 py-3">
            <div className="text-sm">选择文件夹</div>
            <div className="text-xs flex items-center gap-1 ml-2 text-ink/50" onClick={() => goto('-1')}>
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
                  <FileThumb data={data} className={classNames('mb-1 h-16 w-16 p-1 rounded-sm')} />
                  <div className={classNames('w-16 rounded-sm px-1')}>
                    <div className="truncate text-center text-xs">
                      {data.name}
                    </div>
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
              <Button variant="outline" size="sm">
                取消
              </Button>
            </Dialog.Close>
            <Dialog.Close asChild onClick={() => onConfirm(currentExplorerItem)}>
              <Button variant="accent" size="sm">
                选择当前文件夹
              </Button>
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
