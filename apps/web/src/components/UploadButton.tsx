'use client'
import { open } from '@tauri-apps/api/dialog'
import { HTMLAttributes, PropsWithChildren, useCallback } from 'react'
import { cn } from '@/lib/utils'

type Props = {
  onSelectFiles: (fileFullPath: string[]) => void
}

const TauriUploadButton: React.FC<
  PropsWithChildren<Props> & HTMLAttributes<HTMLLabelElement>
> = ({ onSelectFiles, children, className, ...props }) => {
  let handleClick = useCallback(async () => {
    const results = await open({
      title: "Select files to import",
      directory: false,
      multiple: true,
      filters: [
        {
          name: 'Video',
          extensions: ['mp4', 'mov', 'avi', 'mkv'],
        },
        {
          name: 'Images',
          extensions: ['jpg', 'jpeg', 'png', 'gif', "webp"]
        }
      ],
    })
    console.log('tauri selected file:', results)
    if (results && results.length) {
      const fileFullPaths = results as string[]
      onSelectFiles(fileFullPaths)
    } else {
      return null
    }
  }, [onSelectFiles])

  return (
    <form className="block appearance-none">
      <label htmlFor="file-input-select-new-asset" className={cn("cursor-default", className)} onClick={() => handleClick()}>
        {children ? children : <div className='text-xs'>Import medias</div>}
      </label>
    </form>
  )
}

const WebUploadButton: React.FC<
  PropsWithChildren<Props> & HTMLAttributes<HTMLLabelElement>
> = ({ onSelectFiles, children, className, ...props }) => {
  /**
   * 浏览器里选择的文件拿不到全路径，只能拿到文件内容
   * 所以用了这个方法，选择一个 .list 文件，里面包含要上传的视频的路径，一行一个，比如
   * ```files.list
   * /Users/xxx/Downloads/a.mp4
   * /Users/xxx/Downloads/b.mp4
   * /Users/xxx/Downloads/c/c.mp4
   * ```
   */
  const onFileInput = useCallback((e: React.FormEvent<HTMLInputElement>) => {
    const files = e.currentTarget.files
    if (!files || files.length === 0) {
      return
    }
    console.log('form input selected file:', files)
    const file = files[0]
    e.currentTarget.value = ''  // 重置 inputvalue 以便下次选择同一个文件时触发 onInput 事件
    const reader = new FileReader()
    reader.onload = function () {
      const text = (reader.result ?? '').toString()
      console.log(text)
      const fileFullPaths = text.split('\n').map((s) => s.trim()).filter((s) => !!s.length)
      // console.log(fileFullPaths)
      onSelectFiles(fileFullPaths)
    }
    reader.readAsText(file)
  }, [onSelectFiles])

  return (
    <form className="block appearance-none">
      <label htmlFor="file-input-select-new-asset" className={cn("cursor-default", className)}>
        {children ? children : <div className='text-xs'>Import medias</div>}
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

export default function UploadButton({
  children, onSelectFiles, ...props
}: PropsWithChildren<Props> & HTMLAttributes<HTMLLabelElement>) {
  if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
    return <TauriUploadButton onSelectFiles={onSelectFiles} {...props}>{children}</TauriUploadButton>
  } else {
    return <WebUploadButton onSelectFiles={onSelectFiles} {...props}>{children}</WebUploadButton>
  }
}
