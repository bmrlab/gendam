// import { SUPPORTED_CONTENT_TYPES } from '@/constants'
import { cn } from '@/lib/utils'
import { open } from '@tauri-apps/api/dialog'
import { HTMLAttributes, PropsWithChildren, useCallback } from 'react'

type Props = {
  onSelectFiles?: (fileFullPath: File[]) => void
  onSelectFilePaths?: (fileFullPath: string[]) => void
}

const TauriUploadButton: React.FC<PropsWithChildren<Props> & HTMLAttributes<HTMLLabelElement>> = ({
  onSelectFilePaths,
  children,
  className,
  ...props
}) => {
  let handleClick = useCallback(async () => {
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
    if (results && results.length) {
      const fileFullPaths = results as string[]
      onSelectFilePaths && onSelectFilePaths(fileFullPaths)
    } else {
      return null
    }
  }, [onSelectFilePaths])

  return (
    <form className="block appearance-none">
      <label
        htmlFor="file-input-select-new-asset"
        className={cn('cursor-default', className)}
        onClick={() => handleClick()}
      >
        {children ? children : <div className="text-xs">Import medias</div>}
      </label>
    </form>
  )
}

// Deprecated
const WebUploadButtonFromListFile: React.FC<PropsWithChildren<Props> & HTMLAttributes<HTMLLabelElement>> = ({
  onSelectFilesPaths,
  children,
  className,
  ...props
}) => {
  /**
   * 浏览器里选择的文件拿不到全路径，只能拿到文件内容
   * 所以用了这个方法，选择一个 .list 文件，里面包含要上传的视频的路径，一行一个，比如
   * ```files.list
   * /Users/xxx/Downloads/a.mp4
   * /Users/xxx/Downloads/b.mp4
   * /Users/xxx/Downloads/c/c.mp4
   * ```
   */
  const onFileInput = useCallback(
    (e: React.FormEvent<HTMLInputElement>) => {
      const files = e.currentTarget.files
      if (!files || files.length === 0) {
        return
      }
      console.log('form input selected file:', files)
      const file = files[0]
      e.currentTarget.value = '' // 重置 inputvalue 以便下次选择同一个文件时触发 onInput 事件
      const reader = new FileReader()
      reader.onload = function () {
        const text = (reader.result ?? '').toString()
        console.log(text)
        const fileFullPaths = text
          .split('\n')
          .map((s) => s.trim())
          .filter((s) => !!s.length)
        // console.log(fileFullPaths)
        onSelectFilesPaths && onSelectFilesPaths(fileFullPaths)
      }
      reader.readAsText(file)
    },
    [onSelectFilesPaths],
  )

  return (
    <form className="block appearance-none">
      <label htmlFor="file-input-select-new-asset" className={cn('cursor-default', className)}>
        {children ? children : <div className="text-xs">Import medias</div>}
      </label>
      <input
        type="file"
        id="file-input-select-new-asset"
        className="hidden"
        multiple={false}
        accept=".list"
        onInput={onFileInput}
      />
      {/* <button type="submit">上传文件</button> */}
    </form>
  )
}

const WebUploadButton: React.FC<PropsWithChildren<Props> & HTMLAttributes<HTMLLabelElement>> = ({
  onSelectFiles,
  children,
  className,
  ...props
}) => {
  const onFileInput = useCallback(
    (e: React.FormEvent<HTMLInputElement>) => {
      if (e.currentTarget.files) {
        const files: File[] = []
        const _files = e.currentTarget.files
        for (let i = 0; i < _files.length; i++) {
          files.push(_files[i])
        }
        console.log('form input selected file:', files)
        onSelectFiles && onSelectFiles(files)
      }
      e.currentTarget.value = '' // 重置 inputvalue 以便下次选择同一个文件时触发 onInput 事件
    },
    [onSelectFiles],
  )

  return (
    <form className="block appearance-none">
      <label htmlFor="file-input-select-new-asset" className={cn('cursor-default', className)}>
        {children ? children : <div className="text-xs">Import medias</div>}
      </label>
      <input
        type="file"
        id="file-input-select-new-asset"
        className="hidden"
        multiple={true}
        accept="*"
        onInput={onFileInput}
      />
    </form>
  )
}

export default function UploadButton({
  children,
  onSelectFiles,
  ...props
}: PropsWithChildren<Props> & HTMLAttributes<HTMLLabelElement>) {
  if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
    return (
      <TauriUploadButton onSelectFiles={onSelectFiles} {...props}>
        {children}
      </TauriUploadButton>
    )
  } else {
    return (
      <WebUploadButton onSelectFiles={onSelectFiles} {...props}>
        {children}
      </WebUploadButton>
    )
  }
}
