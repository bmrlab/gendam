'use client'
import { useExplorerContext } from '@/Explorer/hooks'
import Icon from '@/components/Icon'
import UploadButton from '@/components/UploadButton'
import { rspc } from '@/lib/rspc'
import { useRouter } from 'next/navigation'
import { useCallback, useState } from 'react'
import TitleDialog from './TitleDialog'
import { useUploadQueueStore } from '@/store/uploadQueue'
import Viewport from '@/components/Viewport'
import PageNav from '@/components/PageNav'
import { useInspector } from './Inspector'

export default function Header() {
  const router = useRouter()
  const explorer = useExplorerContext()
  const uploadQueueStore = useUploadQueueStore()

  const inspector = useInspector()
  const createPathMut = rspc.useMutation(['assets.create_file_path'])

  // const goToDir = useCallback(
  //   (dirName: string) => {
  //     if (!explorer.parentPath) {
  //       return
  //     }
  //     let newPath = explorer.parentPath
  //     if (dirName === '-1') {
  //       newPath = newPath.replace(/(.*\/)[^/]+\/$/, '$1')
  //     } else {
  //       newPath += dirName + '/'
  //     }
  //     router.push('/explorer?dir=' + newPath)
  //   },
  //   [explorer, router],
  // )

  let handleSelectFiles = useCallback(
    (fileFullPaths: string[]) => {
      if (explorer.parentPath) {
        for (let fileFullPath of fileFullPaths) {
          uploadQueueStore.enqueue({
            path: explorer.parentPath,
            localFullPath: fileFullPath,
          })
        }
      }
    },
    [explorer.parentPath, uploadQueueStore],
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
        materializedPath: explorer.parentPath,
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
      <Viewport.Toolbar>
        <PageNav title={explorer.parentPath === '/' ? '全部' : explorer.parentPath} />
        <div className="ml-auto" />
        <div className="mr-8 flex select-none items-center">
          <div className="cursor-pointer px-2 py-1 text-sm" onClick={() => handleCreateDir()}>
            添加文件夹
          </div>
          <UploadButton onSelectFiles={handleSelectFiles} />
        </div>
        <div className="flex items-center gap-0.5 justify-self-end text-[#676C77]">
          <div
            className="h-6 w-[28px] cursor-pointer rounded px-1.5 py-1 hover:bg-toolbar-hover"
            onClick={() => explorer.settings.update({ layout: 'grid' })}
          >
            <Icon.grid className="size-4" />
          </div>
          <div
            className="h-6 w-[28px] cursor-pointer rounded px-1.5 py-1 hover:bg-toolbar-hover"
            onClick={() => explorer.settings.update({ layout: 'list' })}
          >
            <Icon.list className="size-4" />
          </div>
          <div className="w-px h-6 mx-1 bg-app-line"></div>
          <div
            className="h-6 w-[28px] cursor-pointer rounded px-1.5 py-1 hover:bg-toolbar-hover"
            onClick={() => inspector.setShow(!inspector.show)}
          >
            <Icon.viewVertical className="size-4" />
          </div>
          {/* <div className="h-6 w-[28px] cursor-pointer rounded px-1.5 py-1 hover:bg-toolbar-hover">
            <Icon.column className="size-4" />
          </div> */}
        </div>
      </Viewport.Toolbar>
      {titleInputDialogVisible && <TitleDialog onConfirm={onConfirmTitleInput} onCancel={onCancelTitleInput} />}
    </>
  )
}
