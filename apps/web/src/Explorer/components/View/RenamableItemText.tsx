'use client'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
import { HTMLAttributes, useCallback, useEffect, createRef } from 'react'
import classNames from 'classnames'

export default function RenamableItemText({
  data, className
}: HTMLAttributes<HTMLDivElement> & { data: ExplorerItem }) {
  const explorerStore = useExplorerStore()
  const explorer = useExplorerContext()
  const renameMut = rspc.useMutation(['assets.rename_file_path'])
  const inputRef = createRef<HTMLInputElement>()

  useEffect(() => {
    if (inputRef.current) {
      const el = inputRef.current
      el.value = data.name
      // context menu 有个 transition, 要过大约 200ms 才消失, 如果提前 focus input 会立马 blur
      setTimeout(() => {
        el.focus()
        el.select()
      }, 200)
    }
  }, [inputRef, data.name])

  const handleInputSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault()
      if (!inputRef.current?.value) {
        return
      }
      explorerStore.setIsRenaming(false)
      // explorerStore.reset()
      renameMut.mutate({
        id: data.id,
        materializedPath: data.materializedPath,
        isDir: data.isDir,
        oldName: data.name,
        newName: inputRef.current.value,
      })
    },
    [inputRef, explorerStore, renameMut, data],
  )

  return (
    <form className={classNames("w-full", className)} onSubmit={handleInputSubmit}>
      <input
        ref={inputRef}
        className="block w-full rounded-md outline-none text-ink bg-app border-2 border-blue-600 px-2 py-1 text-xs"
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
