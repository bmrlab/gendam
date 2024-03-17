'use client'
import { useExplorerContext } from '@/Explorer/hooks/useExplorerContext'
import GridView from '@/Explorer/components/View/GridView'

export default function Explorer() {
  const explorer = useExplorerContext()

  if (!explorer.items || explorer.items.length === 0) {
    return (
      <div className="flex h-full items-center justify-center">
        <p className="text-neutral-400 text-sm">当前文件夹为空</p>
      </div>
    )
  }

  return (
    <>
      <GridView items={explorer.items}></GridView>
    </>
  )
}
