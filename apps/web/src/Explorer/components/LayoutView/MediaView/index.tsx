'use client'
import { ExtractExplorerItem } from '@/Explorer/types'
import { useMemo } from 'react'
import Folders from './Folders'
import Medias from './Medias'

export default function MediaView({ items }: { items: ExtractExplorerItem<'FilePath'>[] }) {
  const [folders, medias] = useMemo(() => {
    return items.reduce<[ExtractExplorerItem<'FilePath'>[], ExtractExplorerItem<'FilePath'>[]]>(
      ([folders, medias]: [ExtractExplorerItem<'FilePath'>[], ExtractExplorerItem<'FilePath'>[]], item) => {
        if (item.filePath.isDir) {
          folders.push(item)
        } else {
          medias.push(item)
        }
        // FIXME don't know why ts throw error on this type
        return [folders, medias] as [ExtractExplorerItem<'FilePath'>[], ExtractExplorerItem<'FilePath'>[]]
      },
      [[], []],
    )
  }, [items])

  return (
    <>
      {/* 暂时隐藏 folders */}
      {folders.length > 0 && (
        <>
          <Folders items={folders} />
          <div className="bg-app-line my-2 h-px"></div>
        </>
      )}
      <Medias items={medias} />
    </>
  )
}
