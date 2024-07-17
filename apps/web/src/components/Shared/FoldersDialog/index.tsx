import { type FilePath } from '@/lib/bindings'
import Icon from '@gendam/ui/icons'
import { client, rspc } from '@/lib/rspc'
import { toast } from 'sonner'
import { Folder_Light } from '@gendam/assets/images'
import { Button } from '@gendam/ui/v2/button'
import { Dialog } from '@gendam/ui/v2/dialog'
import { RSPCError } from '@rspc/client'
import classNames from 'classnames'
import Image from 'next/image'
import { useCallback, useEffect, useState } from 'react'
import { useFoldersDialog } from './store'
import { useTranslation } from 'react-i18next'

export default function FoldersDialog() {
  const { t } = useTranslation()
  const foldersDialog = useFoldersDialog()
  const [currentPath, setCurrentPath] = useState<string>('/')
  const [selectedFolder, setSelectedFolder] = useState<FilePath | null>(null)

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

  useEffect(() => {
    if (!foldersDialog.open) {
      // 关闭以后清空状态
      setCurrentPath('/')
      setSelectedFolder(null)
    }
  }, [foldersDialog.open])

  const [currentFilePath, setCurrentFilePath] = useState<FilePath | null>(null)

  useEffect(() => {
    const match = currentPath.match(/^((\/[^/]+)*\/)([^/]+)\/$/)
    if (match) {
      const materializedPath = match[1]
      const name = match[3]
      client
        .query(['assets.get', { materializedPath, name }])
        .then((data) => {
          setCurrentFilePath(data)
        })
        .catch((error) => {
          toast.error(`Error fetch folder ${currentPath}`, {
            description: error.message,
          })
        })
    } else {
      setCurrentFilePath(null)
    }
  }, [setCurrentFilePath, currentPath])

  const selectFolder = useCallback((data: FilePath|null) => {
    setSelectedFolder(data)
  }, [])

  const gotoFolder = useCallback(
    (data: FilePath | '-1') => {
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
    <Dialog.Root open={foldersDialog.open} onOpenChange={(open) => foldersDialog.setOpen(open)}>
      <Dialog.Portal>
        <Dialog.Overlay onClick={(e) => e.stopPropagation()} />
        <Dialog.Content onClick={(e) => e.stopPropagation()} className="w-[40rem]">
          <div className="flex items-center justify-start border-b border-app-line px-4 py-3">
            <div className="text-sm">{t('shared.chooseFolder')}</div>
            <Button
              variant="ghost"
              size="xs"
              className="ml-2 gap-1 px-1"
              onClick={() => gotoFolder('-1')}>
              <Icon.ArrowUpLeft className="h-4 w-4" />
              <span>{t('shared.back')}</span>
            </Button>
          </div>
          <div
            className="flex h-80 flex-wrap content-start items-start justify-start gap-4 overflow-auto p-4"
            onClick={() => selectFolder(null)}
          >
            {dirs && dirs.length > 0 ? (
              dirs.map((data) => (
                <div
                  key={data.id}
                  onDoubleClick={() => gotoFolder(data)}
                  onClick={(e) => {
                    e.stopPropagation()
                    selectFolder(data)
                  }}
                  className="flex cursor-default select-none flex-col items-center justify-start"
                >
                  <div className={classNames(
                    "mb-1 h-16 w-16 rounded p-1",
                    selectedFolder?.id === data.id ? "bg-app-hover" : null
                  )}>
                    <div className="overflow-hidden relative h-full w-full">
                      <Image src={Folder_Light} alt="folder" priority fill={true} className="object-contain"></Image>
                    </div>
                  </div>
                  <div className={classNames(
                    "w-16 rounded px-1",
                    selectedFolder?.id === data.id ? "bg-accent text-white" : null
                  )}>
                    <div className="truncate text-center text-xs">{data.name}</div>
                  </div>
                </div>
              ))
            ) : (
              <div className="w-full py-8 text-center text-xs text-ink/50">{t('shared.noFolders')}</div>
            )}
          </div>
          <div className="border-app-line flex items-center justify-end gap-2 border-t px-4 py-2 text-sm">
            <div className="h-6 w-6">
              <Image src={Folder_Light} alt="folder" priority></Image>
            </div>
            <div className="text-xs">{currentPath}</div>
            <div className="mr-auto"></div>
            <Dialog.Close asChild>
              <Button variant="outline" size="sm">{t('shared.cancel')}</Button>
            </Dialog.Close>
            <Dialog.Close asChild onClick={() => {
              if (selectedFolder) {
                foldersDialog.confirm(selectedFolder)
              } else {
                foldersDialog.confirm(currentFilePath)
              }
            }}>
              <Button variant="accent" size="sm">
                {t('shared.choose', {folder: selectedFolder ? `folder "${selectedFolder.name}"` : t('shared.current')})}
              </Button>
            </Dialog.Close>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
