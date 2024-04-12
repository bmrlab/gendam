'use client'
import { useExplorerContext } from '@/Explorer/hooks'
import PageNav from '@/components/PageNav'
import UploadButton from '@/components/UploadButton'
import Viewport from '@/components/Viewport'
import { rspc } from '@/lib/rspc'
import { useUploadQueueStore } from '@/store/uploadQueue'
import Icon from '@muse/ui/icons'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { useCallback, useState } from 'react'
import SearchForm from '../../search/SearchForm'  // TODO: 这样不大好，应该是一个公共组件
import { useInspector } from './Inspector'
import TitleDialog from './TitleDialog'

export default function Header() {
  const router = useRouter()
  const explorer = useExplorerContext()
  const uploadQueueStore = useUploadQueueStore()

  const inspector = useInspector()
  const createPathMut = rspc.useMutation(['assets.create_file_path'])

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

  const handleCreateDir = useCallback(() => {
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

  const handleSearch = useCallback((text: string, recordType: string) => {
    const search = new URLSearchParams()
    search.set('text', text)
    search.set('recordType', recordType)
    router.push(`/search?${search}`)
  }, [router])

  return (
    <>
      <Viewport.Toolbar className="justify-start">
        <PageNav
          title={explorer.parentPath === '/' ? '全部' : explorer.parentPath}
          className="w-1/3"
        />
        <SearchForm
          initialSearchPayload={null}
          onSubmit={(text: string, recordType: string) => handleSearch(text, recordType)}
        />
        <div className="ml-auto"></div>
        {/* <div className="mr-8 flex select-none items-center">
          <div className="cursor-pointer px-2 py-1 text-sm" onClick={() => handleCreateDir()}>
            添加文件夹
          </div>
          <UploadButton onSelectFiles={handleSelectFiles}>上传文件</UploadButton>
        </div> */}
        <div className="text-ink/70 flex items-center gap-1 justify-self-end">
          <div className='hover:bg-toolbar-hover h-6 w-6 rounded p-1' onClick={() => handleCreateDir()}>
            <Icon.FolderOpen className="size-4" />
          </div>
          <div className='hover:bg-toolbar-hover h-6 w-6 rounded p-1'>
            <UploadButton onSelectFiles={handleSelectFiles}>
              <Icon.Upload className="size-4" />
            </UploadButton>
          </div>

          <div className="bg-toolbar-line mx-1 h-4 w-px"></div>

          <div
            className={classNames(
              'hover:bg-toolbar-hover h-6 w-6 rounded p-1',
              explorer.settings.layout === 'grid' && 'bg-toolbar-hover',
            )}
            onClick={() => explorer.settings.update({ layout: 'grid' })}
          >
            <Icon.Grid className="size-4" />
          </div>
          <div
            className={classNames(
              'hover:bg-toolbar-hover h-6 w-6 rounded p-1',
              explorer.settings.layout === 'list' && 'bg-toolbar-hover',
            )}
            onClick={() => explorer.settings.update({ layout: 'list' })}
          >
            <Icon.List className="size-4" />
          </div>
          <div
            className={classNames(
              'hover:bg-toolbar-hover h-6 w-6 rounded p-1',
              explorer.settings.layout === 'media' && 'bg-toolbar-hover',
            )}
            onClick={() => explorer.settings.update({ layout: 'media' })}
          >
            <Icon.Gallery className="size-4" />
          </div>

          <div className="bg-toolbar-line mx-1 h-4 w-px"></div>

          <div
            className={classNames(
              'hover:bg-toolbar-hover h-6 w-6 rounded p-1',
              inspector.show && 'bg-toolbar-hover',
            )}
            onClick={() => inspector.setShow(!inspector.show)}
          >
            <Icon.Column className="size-4" />
          </div>
        </div>
      </Viewport.Toolbar>
      {titleInputDialogVisible && <TitleDialog onConfirm={onConfirmTitleInput} onCancel={onCancelTitleInput} />}
    </>
  )
}

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
