'use client'
import { open } from '@tauri-apps/api/dialog'
import { useCallback } from 'react'

type Props = {
  onSelectFiles: (fileFullPath: string[]) => void
}

const TauriUploadButton: React.FC<Props> = ({ onSelectFiles }) => {
  let handleClick = useCallback(async () => {
    const results = await open({
      directory: false,
      multiple: true,
      filters: [
        {
          name: 'Video',
          extensions: ['mp4', 'mov', 'avi', 'mkv'],
        },
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
    <div>
      <form className="ml-4">
        <label htmlFor="file-input-select-new-asset" className="cursor-pointer text-sm" onClick={() => handleClick()}>
          上传文件
        </label>
      </form>
    </div>
  )
}

const WebUploadButton: React.FC<Props> = ({ onSelectFiles }) => {
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
    const files = (e.target as any)?.files ?? []
    console.log('form input selected file:', files)
    const file = files[0]
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
    <div>
      <form className="ml-4">
        <label htmlFor="file-input-select-new-asset" className="cursor-pointer text-sm">
          上传文件
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
    </div>
  )
}

export default function UploadButton({ onSelectFiles }: Props) {
  if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
    return <TauriUploadButton onSelectFiles={onSelectFiles} />
  } else {
    return <WebUploadButton onSelectFiles={onSelectFiles} />
  }
}
