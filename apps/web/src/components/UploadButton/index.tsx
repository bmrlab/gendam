// import { SUPPORTED_CONTENT_TYPES } from '@/constants'
import { cn } from '@/lib/utils'
import { open } from '@tauri-apps/api/dialog'
import { readDir } from '@tauri-apps/api/fs'
import { HTMLAttributes, PropsWithChildren, useCallback, useId } from 'react'
export { useFileUploadUtils } from './useFileUploadUtils'

type TButtonProps = PropsWithChildren<{
  directory: boolean
  onSelect: (result: UploadButtonResult) => void
}> &
  Omit<HTMLAttributes<HTMLLabelElement>, 'onSelect'>

export type UploadButtonResult =
  | {
      items: (
        | {
            file: File // webview 选择的文件
            relativePath: string // a/b/c 这样的相对路径, 表示文件在所选择的目录下的相对路径
          }
        | {
            fileSystemPath: string // 本地文件系统路径
            relativePath: string // a/b/c 这样的相对路径, 表示文件在所选择的目录下的相对路径
          }
      )[]
      directory: true
    }
  | {
      items: ({ file: File } | { fileSystemPath: string })[]
      directory: false
    }

const traverseDirectory = async (
  dir: string,
): Promise<
  {
    fileSystemPath: string
    relativePath: string
  }[]
> => {
  // TODO: 这里需要把根目录提取出来。
  const dirName = dir.split('/').pop() // 所选文件夹的名称
  if (!dirName) {
    throw new Error('dirName is empty')
  }
  const entries = await readDir(dir, { recursive: true })
  const queue = entries
    .filter((entry) => !!entry.name && !entry.name.startsWith('.')) // 过滤掉隐藏文件
    .map((entry) => ({
      entry,
      relativePath: dirName,
    }))
  const result = []
  while (true) {
    const item = queue.shift()
    if (!item) {
      break
    } else if (item.entry.children) {
      const relativePath = item.relativePath + '/' + item.entry.name
      queue.push(
        ...item.entry.children
          .filter((entry) => !!entry.name && !entry.name.startsWith('.'))
          .map((entry) => ({ entry, relativePath })),
      )
    } else {
      result.push({
        fileSystemPath: item.entry.path,
        relativePath: item.relativePath,
      })
    }
  }
  return result
}

const TauriUploadButton: React.FC<TButtonProps> = ({
  onSelect,
  directory,
  children,
  className,
  // ...props
}) => {
  let handleClickForFolderSelect = useCallback(async () => {
    const result = await open({
      title: 'Select a folder to import',
      directory: true,
      multiple: false,
    })
    console.log('tauri selected directory:', result)
    if (result && typeof result === 'string') {
      const items = await traverseDirectory(result)
      onSelect({ items, directory: true })
    } else {
      onSelect({ items: [], directory: true })
    }
  }, [onSelect])

  let handleClickForFilesSelect = useCallback(async () => {
    const results = await open({
      title: 'Select files to import',
      directory: false,
      multiple: true,
      filters: [
        // 暂时不限制格式，理论上所有格式都应该支持
        // {
        //   name: 'Supported Content',
        //   extensions: Array.from(SUPPORTED_CONTENT_TYPES),
        // },
      ],
    })
    console.log('tauri selected file:', results)
    if (results && Array.isArray(results) && results.length > 0) {
      const items = results.map((fileSystemPath) => ({
        fileSystemPath,
      }))
      onSelect({ items, directory: false })
    } else {
      onSelect({ items: [], directory: false })
    }
  }, [onSelect])

  return (
    <form className="block appearance-none">
      <label
        htmlFor="file-input-select-new-asset"
        className={cn('cursor-default', className)}
        onClick={() => (directory ? handleClickForFolderSelect() : handleClickForFilesSelect())}
      >
        {children ? children : <div className="text-xs">Import medias</div>}
      </label>
    </form>
  )
}

const WebUploadButton: React.FC<TButtonProps> = ({
  onSelect,
  directory,
  children,
  className,
  // ...props
}) => {
  const uniqueId = useId()
  const inputId = `file-input-select-new-asset-${uniqueId}`

  const handleInputForFolderSelect = useCallback(
    (e: React.FormEvent<HTMLInputElement>) => {
      if (e.currentTarget.files) {
        const files = Array.from(e.currentTarget.files || [])
          .filter((file) => !file.name.startsWith('.')) // 过滤掉隐藏文件
          .filter((file) => !!file.webkitRelativePath) // 过滤掉不支持 webkitRelativePath 的文件
        console.log('form input selected files:', files)
        const items = files.map((file) => {
          const relativePath = file.webkitRelativePath.split('/').slice(0, -1).join('/')
          return { file, relativePath }
        })
        for (let i = 0; i < files.length; i++) {}
        onSelect({ items, directory: true })
      }
      e.currentTarget.value = '' // 重置 inputvalue 以便下次选择同一个文件时触发 onInput 事件
    },
    [onSelect],
  )

  const handleInputForFilesSelect = useCallback(
    (e: React.FormEvent<HTMLInputElement>) => {
      if (e.currentTarget.files) {
        const files = e.currentTarget.files
        console.log('form input selected files:', files)
        const items: {
          file: File
        }[] = []
        for (let i = 0; i < files.length; i++) {
          items.push({ file: files[i] })
        }
        onSelect({ items, directory: false })
      }
      e.currentTarget.value = '' // 重置 inputvalue 以便下次选择同一个文件时触发 onInput 事件
    },
    [onSelect],
  )

  return (
    <form className="block appearance-none">
      <label htmlFor={inputId} className={cn('cursor-default', className)}>
        {children ? children : <div className="text-xs">Import medias</div>}
      </label>
      {directory ? (
        <input
          type="file"
          className="hidden"
          id={inputId}
          multiple={false}
          {...({ webkitdirectory: '', directory: '' } as any)}
          onInput={handleInputForFolderSelect}
        />
      ) : (
        <input
          type="file"
          className="hidden"
          id={inputId}
          multiple={true}
          accept="*"
          onInput={handleInputForFilesSelect}
        />
      )}
    </form>
  )
}

export default function UploadButton({ children, onSelect, ...props }: TButtonProps) {
  if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
    return (
      <TauriUploadButton onSelect={onSelect} {...props}>
        {children}
      </TauriUploadButton>
    )
  } else {
    return (
      <WebUploadButton onSelect={onSelect} {...props}>
        {children}
      </WebUploadButton>
    )
  }
}
