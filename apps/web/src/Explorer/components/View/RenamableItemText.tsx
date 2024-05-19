'use client'
// import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { type ExplorerItem } from '@/Explorer/types'
import { rspc, queryClient } from '@/lib/rspc'
import { HTMLAttributes, useCallback, useEffect, useRef } from 'react'
import classNames from 'classnames'

type FilePathExplorerItem = Extract<ExplorerItem, { type: "FilePath" }>

export default function RenamableItemText({
  data, className
}: HTMLAttributes<HTMLDivElement> & { data: FilePathExplorerItem }) {
  const explorerStore = useExplorerStore()
  // const explorer = useExplorerContext()
  const renameMut = rspc.useMutation(['assets.rename_file_path'])
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (inputRef.current) {
      const el = inputRef.current
      el.value = data.filePath.name
      // context menu 有个 transition, 要过大约 200ms 才消失, 如果提前 focus input 会立马 blur
      setTimeout(() => {
        el.focus()
        el.select()
      }, 200)
    }
  }, [inputRef, data.filePath.name])

  const handleInputSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault()
      if (!inputRef.current?.value) {
        return
      }
      explorerStore.setIsRenaming(false)
      // explorerStore.reset()
      /**
       * @todo 这里 mutate({}, { onSuccess }) 里面的 onSuccess 不会被触发,
       * 但是 uploadqueue 里面可以, 太奇怪了
       */
      try {
        await renameMut.mutateAsync({
          id: data.filePath.id,
          materializedPath: data.filePath.materializedPath,
          isDir: data.filePath.isDir,
          oldName: data.filePath.name,
          newName: inputRef.current.value,
        })
      } catch (error) {}
      queryClient.invalidateQueries({
        queryKey: ['assets.list', { materializedPath: data.filePath.materializedPath }]
      })
    },
    [explorerStore, renameMut, data],
  )

  return (
    <form className={classNames("w-full")} onSubmit={handleInputSubmit}>
      <input
        ref={inputRef}
        className={classNames(
          "block w-full rounded-md outline-none text-ink bg-app border-2 border-blue-600 px-2 py-1 text-xs",
          className
        )}
        type="text"
        onClick={(e) => e.stopPropagation()}
        onDoubleClick={(e) => e.stopPropagation()}
        onBlur={() => {
          explorerStore.setIsRenaming(false)
          console.log('on blur, but do nothing, press enter to submit')
        }}
      />
    </form>
  )
}
