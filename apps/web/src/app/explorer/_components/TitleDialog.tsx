import { useCallback, useRef } from 'react'

const TitleDialog: React.FC<{
  onConfirm: (title: string) => void
  onCancel: () => void
}> = ({ onConfirm, onCancel }) => {
  const inputRef = useRef<HTMLInputElement>(null)
  const handleSearch = useCallback(
    (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()
      const keyword = inputRef.current?.value
      if (!keyword) return
      onConfirm(keyword)
    },
    [onConfirm],
  )
  return (
    <div
      className="fixed left-0 top-0 z-20 flex h-full w-full items-center justify-center bg-neutral-50/50"
      onClick={() => onCancel()}
    >
      <form
        className="block w-96 rounded-md border border-neutral-100 bg-white/90 p-6 shadow"
        onSubmit={handleSearch}
        onClick={(e) => e.stopPropagation()}
      >
        <div>输入名称</div>
        <input
          ref={inputRef}
          type="text"
          className="my-4 block w-full rounded-md bg-neutral-100 px-4 py-2 text-sm text-black"
        />
        <button className="block w-full rounded-md bg-blue-500 p-2 text-center text-sm text-white" type="submit">
          确认
        </button>
      </form>
    </div>
  )
}

export default TitleDialog
