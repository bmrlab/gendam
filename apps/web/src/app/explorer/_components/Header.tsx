'use client'
import { useExplorerContext } from '@/Explorer/hooks'
import { useInspector } from '@/components/Inspector'
import PageNav from '@/components/PageNav'
import UploadButton, { type UploadButtonResult, useFileUploadUtils } from '@/components/UploadButton'
import Viewport from '@/components/Viewport'
import { useClipboardPaste } from '@/hooks/useClipboardPaste'
import { useFileDrop } from '@/hooks/useFileDrop'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import { DropdownMenu } from '@gendam/ui/v2/dropdown-menu'
import classNames from 'classnames'
import { useRouter } from 'next/navigation'
import { useCallback, useEffect, useRef, useState } from 'react'
import SearchForm, { type SearchFormRef } from '../../search/SearchForm' // TODO: 这样不大好，应该是一个公共组件
import TitleDialog, { useTitleDialog } from './TitleDialog'
// import UrlImportDialog, { useUrlImportDialog } from './UrlImportDialog'

export default function Header() {
  const titleDialog = useTitleDialog()
  // const urlImportDialog = useUrlImportDialog()
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

  const { handleSelectEventOfUploadButton } = useFileUploadUtils()

  const { filesDropped, setFilesDropped } = useFileDrop()
  useEffect(() => {
    if (filesDropped.length > 0) {
      setFilesDropped([]) // 要立即清空，不然 handleSelectEventOfUploadButton 因为 materializedPath 变化会再次触发同一批文件
      if (explorer.materializedPath && filesDropped.length > 0) {
        handleSelectEventOfUploadButton(explorer.materializedPath, {
          items: filesDropped.map((fileSystemPath) => ({ fileSystemPath })),
          directory: false,
        })
      }
    }
  }, [explorer.materializedPath, filesDropped, setFilesDropped, handleSelectEventOfUploadButton])

  const { filesPasted, setFilesPasted } = useClipboardPaste()
  useEffect(() => {
    if (filesPasted.length > 0) {
      setFilesPasted([]) // 要立即清空，不然 handleSelectEventOfUploadButton 因为 materializedPath 变化会再次触发同一批文件
      if (explorer.materializedPath && filesPasted.length > 0) {
        handleSelectEventOfUploadButton(explorer.materializedPath, {
          items: filesPasted.map((file) => ({ file })),
          directory: false,
        })
      }
    }
  }, [explorer.materializedPath, filesPasted, setFilesPasted, handleSelectEventOfUploadButton])

  const UploadActions = () => {
    const [uploadActionsOpen, setUploadActionsOpen] = useState(false)
    const onSelect = (result: UploadButtonResult) => {
      if (explorer.materializedPath) {
        handleSelectEventOfUploadButton(explorer.materializedPath, result)
      }
      // 在 DropdownMenu.Item 上先调用 e.preventDefault() 以防止在选择此项时关闭下拉菜单
      // 需要选择文件了以后再关闭 menu，不然 input 会被提前 unmount，导致 oninput 无法触发
      setUploadActionsOpen(false)
    }
    return (
      <DropdownMenu.Root open={uploadActionsOpen} onOpenChange={setUploadActionsOpen}>
        <DropdownMenu.Trigger asChild>
          <Button variant="ghost" className="h-7 w-7 p-1 transition-none">
            <Icon.Add className="size-4" />
          </Button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          {/* portals the content part into document.body, to avoid z-index issues */}
          <DropdownMenu.Content align="end">
            <DropdownMenu.Item onSelect={() => titleDialog.setOpen(true)}>
              <Icon.FolderAdd className="size-4" />
              <span>Create Folder</span>
            </DropdownMenu.Item>
            <UploadButton directory={false} onSelect={onSelect}>
              <DropdownMenu.Item onSelect={(e) => e.preventDefault()}>
                <Icon.Upload className="size-4" />
                <span>Import Files</span>
              </DropdownMenu.Item>
            </UploadButton>
            <UploadButton directory={true} onSelect={onSelect}>
              <DropdownMenu.Item onSelect={(e) => e.preventDefault()}>
                <Icon.Upload className="size-4" />
                <span>Import Folder</span>
              </DropdownMenu.Item>
            </UploadButton>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
    )
  }

  return (
    <>
      <Viewport.Toolbar className="relative">
        <PageNav title={explorer.materializedPath === '/' ? 'Library' : explorer.materializedPath} />
        <div className="absolute left-1/3 w-1/3">
          <SearchForm ref={searchFormRef} onSubmit={() => onSearchFormSubmit()} />
        </div>
        <div className="ml-auto"></div>
        <div className="text-ink/70 flex items-center gap-1 justify-self-end">
          <UploadActions />

          {/* <Button variant="ghost" size="sm" className="h-7 w-7 p-1 transition-none" asChild>
            加上 asChild 不使用 native button, 因为里面是个 form, native button 可能会触发 form submit
            <UploadButton onSelectFilePaths={handleSelectFilePaths} onSelectFiles={handleSelectFiles}>
              <Icon.Upload className="size-4" />
            </UploadButton>
          </Button> */}

          {/* <Button
            variant="ghost"
            size="sm"
            className="h-7 w-7 p-1 transition-none"
            onClick={() => urlImportDialog.setOpen(true)}
          >
            <Icon.Link2 className="size-4" />
          </Button> */}

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
            onClick={() => {
              const show = !inspector.show
              inspector.setShow(show)
              explorer.settings.update({ inspectorShow: show }) // Header 上的按钮会保存设置
            }}
          >
            <Icon.Sidebar className="size-4" />
          </Button>
        </div>
      </Viewport.Toolbar>
      <TitleDialog />
      {/* <UrlImportDialog /> */}
    </>
  )
}
