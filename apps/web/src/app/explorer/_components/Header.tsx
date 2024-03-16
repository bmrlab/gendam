'use client'
import UploadButton from '@/components/UploadButton'
import { rspc } from '@/lib/rspc'
import { useCallback, useRef, useState } from 'react'
import TitleDialog from './TitleDialog'
// import { useExplorerContext } from '@/components/Explorer/Context'

export default function Header({ goToDir, parentPath }: { goToDir: (dirName: string) => void; parentPath: string }) {
  // const explorer = useExplorerContext()

  const createPathMut = rspc.useMutation(['assets.create_file_path'])
  const createAssetMut = rspc.useMutation(['assets.create_asset_object'])

  let handleSelectFile = useCallback(
    (fileFullPath: string) => {
      createAssetMut.mutate({
        path: parentPath,
        localFullPath: fileFullPath,
      })
    },
    [createAssetMut, parentPath],
  )

  const [titleInputDialogVisible, setTitleInputDialogVisible] = useState(false)

  let handleCreateDir = useCallback(() => {
    setTitleInputDialogVisible(true)
  }, [setTitleInputDialogVisible])

  const onConfirmTitleInput = useCallback(
    (title: string) => {
      if (!title) {
        return
      }
      createPathMut.mutate({
        path: parentPath,
        name: title,
      })
      setTitleInputDialogVisible(false)
    },
    [createPathMut, parentPath],
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
          {parentPath !== '/' && (
            <div className="cursor-pointer px-2 py-1" onClick={() => goToDir('-1')}>
              ↑
            </div>
          )}
          <div className="ml-2 text-sm">{parentPath === '/' ? '全部' : parentPath}</div>
        </div>
        <div className="flex select-none items-center">
          <div className="cursor-pointer px-2 py-1 text-sm" onClick={() => handleCreateDir()}>
            添加文件夹
          </div>
          <UploadButton onSelectFile={handleSelectFile} />
        </div>
      </div>
      {titleInputDialogVisible && <TitleDialog onConfirm={onConfirmTitleInput} onCancel={onCancelTitleInput}/>}
    </>
  )
}
