'use client'
import { type ExplorerItem } from '@/Explorer/types'
import { useMemo } from 'react'
import Folders from './Folders'
import Medias from './Medias'

export type WithFilePathExplorerItem = Extract<ExplorerItem, { type: "FilePath" | "SearchResult" }>

export default function MediaView({ items }: { items: WithFilePathExplorerItem[] }) {
  const [folders, medias] = useMemo(() => {
    return items.reduce<[WithFilePathExplorerItem[], WithFilePathExplorerItem[]]>(
      ([folders, medias], item) => {
        if (item.filePath.isDir) {
          folders.push(item)
        } else {
          medias.push(item)
        }
        return [folders, medias]
      },
      [[], []],
    )
  }, [items])

  return (
    <>
      {/* 暂时隐藏 folders */}
      {false && folders.length > 0 && (
        <>
          <Folders items={folders} />
          <div className="bg-app-line my-2 h-px"></div>
        </>
      )}
      <Medias items={medias} />
    </>
  )
}
