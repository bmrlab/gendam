'use client'
import { useExplorerContext } from '@/components/Explorer/Context'
import Icon from '@/components/Icon'
import { Folder_Light } from '@muse/assets/icons'
import Image from 'next/image'
import { useMemo } from 'react'

export default function Footer() {
  const explorer = useExplorerContext()
  const folders = useMemo(() => {
    const list = (explorer.parentPath ?? '/').split('/').filter(Boolean)
    list.unshift('home')
    return list
  }, [explorer.parentPath])

  return (
    <div className="flex h-8 items-center justify-start border-t-2 border-neutral-100 px-4 text-xs">
      {folders.map((folder, index) => (
        <div key={index} className="flex items-center">
          <Image src={Folder_Light} alt="folder" priority className="mr-1 h-4 w-4"></Image>
          <div className="text-neutral-500">{folder}</div>
          {index < folders.length - 1 && (
            <div className="mx-1 text-neutral-500">
              <Icon.arrowRight className="h-4 w-4" />
            </div>
          )}
        </div>
      ))}
    </div>
  )
}
