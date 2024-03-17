'use client'
import { useExplorerContext } from '@/Explorer/hooks'
import UploadButton from '@/components/UploadButton'
import { rspc } from '@/lib/rspc'
import { useRouter } from 'next/navigation'
import { useCallback, useState } from 'react'
import TitleDialog from './TitleDialog'

export default function Header() {
  const router = useRouter()
  const explorer = useExplorerContext()

  const createPathMut = rspc.useMutation(['assets.create_file_path'])
  const createAssetMut = rspc.useMutation(['assets.create_asset_object'])

  const goToDir = useCallback(
    (dirName: string) => {
      if (!explorer.parentPath) {
        return
      }
      let newPath = explorer.parentPath
      if (dirName === '-1') {
        newPath = newPath.replace(/(.*\/)[^/]+\/$/, '$1')
      } else {
        newPath += dirName + '/'
      }
      router.push('/explorer?dir=' + newPath)
    },
    [explorer, router],
  )

  let handleSelectFile = useCallback(
    (fileFullPath: string) => {
      if (explorer.parentPath) {
        createAssetMut.mutate({
          path: explorer.parentPath,
          localFullPath: fileFullPath,
        })
      }
    },
    [createAssetMut, explorer],
  )

  const [titleInputDialogVisible, setTitleInputDialogVisible] = useState(false)

  let handleCreateDir = useCallback(() => {
    setTitleInputDialogVisible(true)
  }, [setTitleInputDialogVisible])

  const onConfirmTitleInput = useCallback(
    (title: string) => {
      if (!title || !explorer.parentPath) {
        return
      }
      createPathMut.mutate({
        path: explorer.parentPath,
        name: title,
      })
      setTitleInputDialogVisible(false)
    },
    [createPathMut, explorer],
  )

  const onCancelTitleInput = useCallback(() => {
    setTitleInputDialogVisible(false)
  }, [setTitleInputDialogVisible])

  return (
    <>
      <div className="flex h-12 justify-between border-b border-neutral-100 px-4">
        <div className="flex select-none items-center">
          <div className="px-2 py-1">&lt;</div>
          <div className="px-2 py-1">&gt;</div>
          {explorer.parentPath !== '/' && (
            <div className="cursor-pointer px-2 py-1" onClick={() => goToDir('-1')}>
              ↑
            </div>
          )}
          <div className="ml-2 text-sm">{explorer.parentPath === '/' ? '全部' : explorer.parentPath}</div>
        </div>
        <div className="flex select-none items-center">
          <div className="cursor-pointer px-2 py-1 text-sm" onClick={() => handleCreateDir()}>
            添加文件夹
          </div>
          <UploadButton onSelectFile={handleSelectFile} />
        </div>
      </div>
      {titleInputDialogVisible && <TitleDialog onConfirm={onConfirmTitleInput} onCancel={onCancelTitleInput} />}
    </>
  )
}
