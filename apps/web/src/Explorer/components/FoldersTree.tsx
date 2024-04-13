'use client'
import { Folder_Light } from '@muse/assets/images'
import { create } from 'zustand'
import Image from 'next/image'
import { rspc } from '@/lib/rspc'
import Icon from '@muse/ui/icons'
import { cn } from '@/lib/utils'
import { HTMLAttributes, useCallback, useMemo, useState } from 'react'
import { FilePath } from '@/lib/bindings'
import { useRouter, usePathname, useSearchParams } from 'next/navigation'
import path from 'path'

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

  const { data: subDirs } = rspc.useQuery([
    'assets.list',
    {
      materializedPath: filePath.materializedPath + filePath.name + '/',
      isDir: true,
    },
  ], {
    // enabled: open
  })

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
          className={cn(
            "w-4 h-5 py-1 pl-1 bg-sidebar text-ink/40",
            !subDirs || !subDirs.length ? "invisible" : ""
          )}
          onClick={() => setOpen(!open)}
        >
          <Icon.ArrowRight className={cn(
            "size-3 transition-all duration-200",
            open ? "rotate-90" : "rotate-0"
          )} />
        </div>
        {/* folder icon and name */}
        <div
          className={cn(
            "my-1 pl-1 py-1 pr-2 rounded flex gap-2 items-center justify-start overflow-hidden",
            // selectionState.id === filePath.id ? "bg-sidebar-hover" : ""
            highlight ? "bg-sidebar-hover" : "hover:bg-sidebar-hover"
          )}
          // onDoubleClick={(e) => onDoubleClick(e)}
          // onClick={(e) => onClick(e)}
          onClick={(e) => onDoubleClick(e)}
        >
          <Image src={Folder_Light} alt="folder" priority className="w-5 h-auto"></Image>
          <div className="truncate text-xs">{filePath.name}</div>
        </div>
      </div>
      {/* children */}
      {open ? (
        <div className="ml-7 border-l border-ink/10">
          {subDirs?.map((subFilePath) => (
            <FoldersBlock key={subFilePath.id} filePath={subFilePath} />
          ))}
        </div>
      ) : null }
    </div>
  )
}

export default function FoldersTree({ className }: HTMLAttributes<HTMLDivElement>) {
  // const selectionState = useSelectionState()
  const router = useRouter()
  const { data: dirs } = rspc.useQuery([
    'assets.list',
    {
      materializedPath: '/',
      isDir: true,
    },
  ])

  return (
    <div
      className={cn('bg-sidebar py-2 overflow-auto', className)}
      // onClick={() => selectionState.set(null)}
    >
      <div className="ml-5 text-xs font-medium text-ink/50 mb-2">Folders</div>
      <div className="ml-5 flex items-center justify-start">
        <div
          className={cn(
            "my-1 pl-1 py-1 pr-2 flex items-center justify-start gap-2",
            "hover:bg-sidebar-hover",
          )}
          // onDoubleClick={(e) => router.push('/explorer')}
          onClick={(e) => router.push('/explorer')}
        >
          <Image src={Folder_Light} alt="folder" priority className="w-5 h-auto"></Image>
          <div className="text-xs">Home</div>
        </div>
      </div>
      <div className="ml-3.5">
        {dirs?.map((filePath) => (
          <FoldersBlock key={filePath.id} filePath={filePath} />
        ))}
      </div>
    </div>
  )
}
