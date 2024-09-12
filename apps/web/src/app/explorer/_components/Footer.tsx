'use client'
import Viewport from '@/components/Viewport'
import { useExplorerContext } from '@/Explorer/hooks'
import { Document_Light, Folder_Light } from '@gendam/assets/images'
import Icon from '@gendam/ui/icons'
import Image from 'next/image'
import { useRouter } from 'next/navigation'
import { useCallback, useMemo } from 'react'

export default function Footer() {
  const router = useRouter()
  const explorer = useExplorerContext()
  const folders = useMemo(() => {
    const list = (explorer.materializedPath ?? '/').split('/').filter(Boolean)
    list.unshift('Library')
    return list
  }, [explorer.materializedPath])

  const theFirstSelectedItem = useMemo(() => {
    const first = Array.from(explorer.selectedItems)[0]
    return first && (first.type === 'FilePathDir' || first.type === 'FilePathWithAssetObject') ? first : null
  }, [explorer])

  const goToFolder = useCallback(
    (index: number) => {
      const joined = folders.slice(1, index + 1).join('/')
      const newPath = joined === '' ? '/' : `/${joined}/`
      router.push('/explorer?dir=' + newPath)
    },
    [folders, router],
  )

  return (
    <Viewport.StatusBar>
      {folders.map((folder, index) => (
        <div key={index} className="flex items-center" onDoubleClick={() => goToFolder(index)}>
          <Image src={Folder_Light} alt="folder" priority className="mr-1 h-4 w-4"></Image>
          <div className="text-xs text-neutral-500">{folder}</div>
          {index < folders.length - 1 && (
            <div className="mx-1 text-neutral-500">
              <Icon.ArrowRight className="h-4 w-4" />
            </div>
          )}
        </div>
      ))}
      {theFirstSelectedItem && (
        <>
          <div className="mx-1 text-neutral-500">
            <Icon.ArrowRight className="h-4 w-4" />
          </div>
          {theFirstSelectedItem.type === 'FilePathDir' ? (
            <Image src={Folder_Light} alt="folder" priority className="mr-1 h-4 w-4"></Image>
          ) : (
            <Image src={Document_Light} alt="folder" priority className="mr-1 h-4 w-4"></Image>
          )}
          <div className="text-xs text-neutral-500">{theFirstSelectedItem.filePath.name}</div>
        </>
      )}
    </Viewport.StatusBar>
  )
}
