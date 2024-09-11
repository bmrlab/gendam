'use client'
import ExplorerDroppable from '@/Explorer/components/Draggable/ExplorerDroppable'
import { ExtractExplorerItem } from '@/Explorer/types'
import { FilePath } from '@/lib/bindings'
import { queryClient, rspc } from '@/lib/rspc'
import { cn } from '@/lib/utils'
import { Folder_Light } from '@gendam/assets/images'
import Icon from '@gendam/ui/icons'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import classNames from 'classnames'
import Image from 'next/image'
import { usePathname, useRouter, useSearchParams } from 'next/navigation'
import { HTMLAttributes, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { create } from 'zustand'

interface FoldersTreeState {
  isRenaming: FilePath | null
  setIsRenaming: (isRenaming: FilePath | null) => void
}

const useFoldersTreeStore = create<FoldersTreeState>((set) => ({
  isRenaming: null,
  setIsRenaming: (isRenaming) => set({ isRenaming }),
}))

// 这里先复制一个 RenamableItemText, 因为 Explorer 组件下的 RenamableItemText 绑定了 explorerStore
const RenamableItemText = ({
  data,
  className,
}: HTMLAttributes<HTMLDivElement> & {
  data: ExtractExplorerItem<'FilePath'>
}) => {
  const foldersTreeStore = useFoldersTreeStore()
  const renameMut = rspc.useMutation(['assets.rename_file_path'])
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    if (inputRef.current) {
      const el = inputRef.current
      el.value = data.filePath.name
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
      foldersTreeStore.setIsRenaming(null)
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
        queryKey: ['assets.list', { materializedPath: data.filePath.materializedPath }],
      })
    },
    [foldersTreeStore, renameMut, data],
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
          foldersTreeStore.setIsRenaming(null)
          console.log('on blur, but do nothing, press enter to submit')
        }}
      />
    </form>
  )
}

const FolderItem: React.FC<{ filePath: FilePath }> = ({ filePath }) => {
  const router = useRouter()
  const pathname = usePathname()
  const searchParams = useSearchParams()
  const highlight = useMemo(() => {
    return pathname === '/explorer' && filePath.materializedPath + filePath.name + '/' === searchParams.get('dir')
  }, [filePath.materializedPath, filePath.name, pathname, searchParams])
  const foldersTreeStore = useFoldersTreeStore()

  const onDoubleClick = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      // e.stopPropagation()
      const newPath = filePath.materializedPath + filePath.name + '/'
      router.push('/explorer?dir=' + newPath)
    },
    [filePath.materializedPath, filePath.name, router],
  )

  return (
    <ContextMenu.Root onOpenChange={() => {}}>
      <ContextMenu.Trigger>
        <ExplorerDroppable
          droppable={{
            data: { type: 'FilePath', filePath: filePath },
            region: 'Sidebar',
          }}
        >
          <div
            className={cn(
              'my-1 flex items-center justify-start gap-2 overflow-hidden rounded py-1 pl-1 pr-2',
              // selectionState.id === filePath.id ? "bg-sidebar-hover" : ""
              highlight ? 'bg-sidebar-hover' : 'hover:bg-sidebar-hover',
            )}
            // onDoubleClick={(e) => onDoubleClick(e)}
            // onClick={(e) => onClick(e)}
            onClick={(e) => onDoubleClick(e)}
          >
            <Image src={Folder_Light} alt="folder" priority className="h-auto w-5"></Image>
            {foldersTreeStore.isRenaming?.id === filePath.id ? (
              <RenamableItemText data={{ type: 'FilePath', filePath }} />
            ) : (
              <div className="truncate text-xs">{filePath.name}</div>
            )}
          </div>
        </ExplorerDroppable>
      </ContextMenu.Trigger>
      <ContextMenu.Portal>
        <ContextMenu.Content>
          <ContextMenu.Item onClick={() => foldersTreeStore.setIsRenaming(filePath)}>Rename</ContextMenu.Item>
        </ContextMenu.Content>
      </ContextMenu.Portal>
    </ContextMenu.Root>
  )
}

const FoldersBlock: React.FC<{ filePath: FilePath }> = ({ filePath }) => {
  const [open, setOpen] = useState(false)

  const subDirsQuery = rspc.useQuery(
    [
      'assets.list',
      {
        materializedPath: filePath.materializedPath + filePath.name + '/',
        isDir: true,
      },
    ],
    {
      // enabled: open
    },
  )

  /**
   * 在文件名中确保 10 > 2
   */
  const sortedItems = useMemo(
    () =>
      [...(subDirsQuery.data || [])].sort((a, b) =>
        a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' }),
      ),
    [subDirsQuery.data],
  )

  // const onClick = useCallback(
  //   (e: React.FormEvent<HTMLDivElement>) => {
  //     e.stopPropagation()
  //     selectionState.set(filePath.id)
  //   },
  //   [filePath.id, selectionState],
  // )

  return (
    <div className="-ml-2.5 mr-2">
      {/* folder item */}
      <div className="flex items-center justify-start">
        {/* caret */}
        <div
          className={cn('bg-sidebar text-ink/40 h-5 w-4 py-1 pl-1', !sortedItems.length ? 'invisible' : '')}
          onClick={() => setOpen(!open)}
        >
          <Icon.ArrowRight className={cn('size-3 transition-all duration-200', open ? 'rotate-90' : 'rotate-0')} />
        </div>
        {/* folder icon and name */}
        <FolderItem filePath={filePath} />
      </div>
      {/* children */}
      {open ? (
        <div className="border-ink/10 ml-7 border-l">
          {sortedItems.map((subFilePath) => (
            <FoldersBlock key={subFilePath.id} filePath={subFilePath} />
          ))}
        </div>
      ) : null}
    </div>
  )
}

export default function FoldersTree({ className }: HTMLAttributes<HTMLDivElement>) {
  // const selectionState = useSelectionState()
  const router = useRouter()
  const dirsQuery = rspc.useQuery([
    'assets.list',
    {
      materializedPath: '/',
      isDir: true,
    },
  ])

  /**
   * 在文件名中确保 10 > 2
   */
  const sortedItems = useMemo(
    () =>
      [...(dirsQuery.data || [])].sort((a, b) =>
        a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' }),
      ),
    [dirsQuery.data],
  )

  const createDirMut = rspc.useMutation(['assets.create_dir'])
  const createNewFolder = async () => {
    await createDirMut.mutateAsync({
      materializedPath: '/',
      name: 'untitled',
    })
    queryClient.invalidateQueries({
      queryKey: ['assets.list', { materializedPath: '/' }],
    })
  }

  return (
    <div
      className={cn('bg-sidebar overflow-auto py-2', className)}
      // onClick={() => selectionState.set(null)}
    >
      <div className="text-ink/50 mb-1 ml-5 mr-5 flex items-center justify-between text-xs font-medium">
        <ExplorerDroppable droppable={{ data: { type: 'LibraryRoot' }, region: 'Sidebar' }}>
          <div className="hover:bg-sidebar-hover rounded p-1" onClick={(e) => router.push('/explorer')}>
            Folders
          </div>
        </ExplorerDroppable>
        <div onClick={createNewFolder} className="hover:bg-sidebar-hover rounded p-1">
          <Icon.Add className="size-3" />
        </div>
      </div>
      {/* <div className="ml-5 flex items-center justify-start">
        <div
          className={cn('my-1 flex items-center justify-start gap-2 py-1 pl-1 pr-2', 'hover:bg-sidebar-hover')}
          // onDoubleClick={(e) => router.push('/explorer')}
          onClick={(e) => router.push('/explorer')}
        >
          <Image src={Folder_Light} alt="folder" priority className="h-auto w-5"></Image>
          <div className="text-xs">Home</div>
        </div>
      </div> */}
      <div className="ml-3.5">
        {sortedItems.map((filePath) => (
          <FoldersBlock key={filePath.id} filePath={filePath} />
        ))}
      </div>
    </div>
  )
}
