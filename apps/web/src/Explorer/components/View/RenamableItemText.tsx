'use client'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import { useExplorerStore } from '@/Explorer/store'
import { ExplorerItem } from '@/Explorer/types'
import { rspc } from '@/lib/rspc'
import { useCallback, useEffect, useRef } from 'react'

export default function RenamableItemText({ data }: { data: ExplorerItem }) {
  const explorerStore = useExplorerStore()
  const explorer = useExplorerContext()
  const renameMut = rspc.useMutation(['assets.rename_file_path'])
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (inputRef.current) {
      inputRef.current.focus()
      inputRef.current.value = data.name
      inputRef.current.select()
    }
  }, [inputRef, data])

  const handleInputSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault()
      if (!inputRef.current?.value) {
        return
      }
      if (!explorer.parentPath) {
        // TODO: explorer.parentPath 到这一步不应该是空的，然后 data.id 如果存在，其实可以忽略 parentPath 参数
        return
      }
      explorerStore.setIsRenaming(false)
      // explorerStore.reset()
      renameMut.mutate({
        id: data.id,
        path: explorer.parentPath,
        isDir: data.isDir,
        oldName: data.name,
        newName: inputRef.current.value,
      })
    },
    [explorer.parentPath, explorerStore, renameMut, data.id, data.isDir, data.name],
  )

  return (
    <form className="w-32" onSubmit={handleInputSubmit}>
      <input
        ref={inputRef}
        className="block w-full rounded-sm outline-none border-2 border-blue-600 px-2 py-1 text-center text-xs"
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
