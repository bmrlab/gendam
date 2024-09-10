'use client'
import ExplorerDroppable from '@/Explorer/components/Draggable/ExplorerDroppable'
import { FilePath } from '@/lib/bindings'
import { queryClient, rspc } from '@/lib/rspc'
import { cn } from '@/lib/utils'
import { Folder_Light } from '@gendam/assets/images'
import Icon from '@gendam/ui/icons'
import Image from 'next/image'
import { usePathname, useRouter, useSearchParams } from 'next/navigation'
import { HTMLAttributes, useCallback, useMemo, useState } from 'react'

// interface SelectionState {
//   id: number | null
//   set: (id: number | null) => void
// }

// export const useSelectionState = create<SelectionState>((set) => ({
//   id: null,
//   set: (id) => set({ id }),
// }))

const FoldersBlock: React.FC<{ filePath: FilePath }> = ({ filePath }) => {
  // const selectionState = useSelectionState()
  const router = useRouter()
  const [open, setOpen] = useState(false)
  const pathname = usePathname()
  const searchParams = useSearchParams()
  const highlight = useMemo(() => {
    return pathname === '/explorer' && filePath.materializedPath + filePath.name + '/' === searchParams.get('dir')
  }, [filePath.materializedPath, filePath.name, pathname, searchParams])

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

  const onDoubleClick = useCallback(
    (e: React.FormEvent<HTMLDivElement>) => {
      // e.stopPropagation()
      const newPath = filePath.materializedPath + filePath.name + '/'
      router.push('/explorer?dir=' + newPath)
    },
    [filePath.materializedPath, filePath.name, router],
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
            <div className="truncate text-xs">{filePath.name}</div>
          </div>
        </ExplorerDroppable>
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
        <div className="hover:bg-sidebar-hover rounded p-1" onClick={(e) => router.push('/explorer')}>
          Folders
        </div>
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
