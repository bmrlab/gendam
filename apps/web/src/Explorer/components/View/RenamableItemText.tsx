import { ExtractExplorerItem } from '@/Explorer/types'
import { queryClient, rspc } from '@/lib/rspc'
import classNames from 'classnames'
import { HTMLAttributes, useCallback, useEffect, useRef } from 'react'

export default function RenamableItemText({
  data,
  className,
  onClose,
}: HTMLAttributes<HTMLDivElement> & {
  data: ExtractExplorerItem<'FilePathDir' | 'FilePathWithAssetObject'>
  onClose: () => void
}) {
  const renameMut = rspc.useMutation(['assets.rename_file_path'])
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (inputRef.current) {
      const el = inputRef.current
      el.value = data.filePath.name
      /**
       * context menu 有个 transition，是通过 data-[state=closed]:animate-out 定义的，要过大约 200ms 才消失，
       * 如果提前 focus input 会立马 blur
       * 现在在 ItemContextMenu 的 Content 上加了个
       *   data-[state=closed]:animate-none data-[state=closed]:duration-0
       * 来去掉关闭时候的动画。
       * 这样 timeout 的时间可以少一点，不过测试下来小于 100ms 还是会有问题 (有时候会立马 blur)，应该还有其他动画或者 transition 在影响。
       */
      setTimeout(() => {
        el.focus()
        el.select()
      }, 100)
    }
  }, [inputRef, data.filePath.name])

  const handleInputSubmit = useCallback(
    async (e: React.FormEvent) => {
      e.preventDefault()
      if (!inputRef.current?.value) {
        return
      }
      onClose()
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
        queryKey: ['assets.list', { materializedPath: data.filePath.materializedPath }],
      })
    },
    [onClose, renameMut, data],
  )

  return (
    <form className={classNames('w-full')} onSubmit={handleInputSubmit}>
      <input
        ref={inputRef}
        className={classNames(
          'text-ink bg-app block w-full text-xs',
          // "border-2 border-blue-600",
          'rounded shadow-[inset_0_0_0_1px] shadow-blue-600',
          'border-none px-1 py-1 outline-none',
          className,
        )}
        type="text"
        onClick={(e) => e.stopPropagation()}
        onDoubleClick={(e) => e.stopPropagation()}
        onBlur={() => {
          onClose()
          console.log('on blur, but do nothing, press enter to submit')
        }}
      />
    </form>
  )
}
