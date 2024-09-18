'use client'
import { useExplorerContext } from '@/Explorer/hooks'
import PageNav from '@/components/PageNav'
import UploadButton from '@/components/UploadButton'
import Viewport from '@/components/Viewport'
// import { rspc } from '@/lib/rspc'
import { useInspector } from '@/components/Inspector/store'
import { useUploadQueueStore } from '@/components/UploadQueue/store'
import { useClipboardPaste } from '@/hooks/useClipboardPaste'
import { useFileDrop } from '@/hooks/useFileDrop'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { useCallback, useEffect, useRef } from 'react'
import SearchForm, { type SearchFormRef } from '../../search/SearchForm' // TODO: 这样不大好，应该是一个公共组件
import TitleDialog, { useTitleDialog } from './TitleDialog'
import UrlImportDialog, { useUrlImportDialog } from './UrlImportDialog'

export default function Header() {
  const titleDialog = useTitleDialog()
  const urlImportDialog = useUrlImportDialog()
  const router = useRouter()
  const explorer = useExplorerContext()

  const inspector = useInspector()

  const searchFormRef = useRef<SearchFormRef>(null)
  // const [searchPayload, setSearchPayload] = useState<SearchRequestPayload | null>(null)
  const onSearchFormSubmit = useCallback(() => {
    if (searchFormRef.current) {
      const search = new URLSearchParams()
      const value = searchFormRef.current.getValue()
      if (value) {
        search.set('text', value.text)
        router.push(`/search?${search.toString()}`)
      }
    }
  }, [router])

  const uploadQueueStore = useUploadQueueStore()
  // 只监听 enqueue，监听 uploadQueueStore 数据变化，不然 handleSelectFiles 会一直更新导致后面有个 useEffect 始终被触发
  const enqueue = uploadQueueStore.enqueue
  const handleSelectFiles = useCallback(
    (fileFullPaths: string[]) => {
      // TODO 暂时隐藏
      // const { supportedFiles, unsupportedExtensionsSet } = filterFiles(fileFullPaths)
      // if (Array.from(unsupportedExtensionsSet).length > 0) {
      //   toast.error(`Unsupported file types: ${Array.from(unsupportedExtensionsSet).join(',')}`)
      // }
      const supportedFiles = fileFullPaths
      if (explorer.materializedPath && supportedFiles.length > 0) {
        for (const fileFullPath of fileFullPaths) {
          const name = fileFullPath.split('/').slice(-1).join('')
          enqueue({
            materializedPath: explorer.materializedPath,
            name: name,
            localFullPath: fileFullPath,
          })
        }
      }
    },
    [explorer.materializedPath, enqueue],
  )

  const { filesDropped } = useFileDrop()
  useEffect(() => {
    if (filesDropped.length > 0) {
      handleSelectFiles(filesDropped)
    }
  }, [filesDropped, handleSelectFiles])

  const { filesPasted } = useClipboardPaste()
  useEffect(() => {
    if (filesPasted.length > 0) {
      handleSelectFiles(filesPasted)
    }
  }, [filesPasted, handleSelectFiles])

  return (
    <>
      <Viewport.Toolbar className="relative">
        <PageNav title={explorer.materializedPath === '/' ? 'Library' : explorer.materializedPath} />
        <div className="absolute left-1/3 w-1/3">
          <SearchForm ref={searchFormRef} onSubmit={() => onSearchFormSubmit()} />
        </div>
        <div className="ml-auto"></div>
        <div className="text-ink/70 flex items-center gap-1 justify-self-end">
          <Button
            variant="ghost"
            size="sm"
            className="h-7 w-7 p-1 transition-none"
            onClick={() => titleDialog.setOpen(true)}
          >
            <Icon.FolderAdd className="size-4" />
          </Button>
          <Button variant="ghost" size="sm" className="h-7 w-7 p-1 transition-none" asChild>
            {/* 加上 asChild 不使用 native button, 因为里面是个 form, native button 可能会触发 form submit */}
            <UploadButton onSelectFiles={handleSelectFiles}>
              <Icon.Upload className="size-4" />
            </UploadButton>
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 w-7 p-1 transition-none"
            onClick={() => urlImportDialog.setOpen(true)}
          >
            <Icon.Link2 className="size-4" />
          </Button>

          <div className="bg-toolbar-line mx-1 h-4 w-px"></div>

          <Button
            variant="ghost"
            size="sm"
            className={classNames(
              'h-7 w-7 p-1 transition-none',
              explorer.settings.layout === 'grid' && 'bg-toolbar-hover',
            )}
            onClick={() => explorer.settings.update({ layout: 'grid' })}
          >
            <Icon.Grid className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className={classNames(
              'h-7 w-7 p-1 transition-none',
              explorer.settings.layout === 'list' && 'bg-toolbar-hover',
            )}
            onClick={() => explorer.settings.update({ layout: 'list' })}
          >
            <Icon.List className="size-4" />
          </Button>
          <Button
            variant="ghost"
            size="sm"
            className={classNames(
              'h-7 w-7 p-1 transition-none',
              explorer.settings.layout === 'media' && 'bg-toolbar-hover',
            )}
            onClick={() => explorer.settings.update({ layout: 'media' })}
          >
            <Icon.SelfAdapting className="size-4" />
          </Button>

          <div className="bg-toolbar-line mx-1 h-4 w-px"></div>

          <Button
            variant="ghost"
            size="sm"
            className={classNames('h-7 w-7 p-1 transition-none', inspector.show && 'bg-toolbar-hover')}
            onClick={() => inspector.setShow(!inspector.show)}
          >
            <Icon.Sidebar className="size-4" />
          </Button>
        </div>
      </Viewport.Toolbar>
      <TitleDialog />
      <UrlImportDialog />
    </>
  )
}
