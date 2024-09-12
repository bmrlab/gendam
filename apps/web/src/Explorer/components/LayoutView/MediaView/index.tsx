'use client'
import { ExtractExplorerItem } from '@/Explorer/types'
import { useMemo } from 'react'
import Folders from './Folders'
import Medias from './Medias'

export default function MediaView({
  items,
}: {
  items: ExtractExplorerItem<'FilePathDir' | 'FilePathWithAssetObject'>[]
}) {
  const [folders, medias] = useMemo(() => {
    return items.reduce<[ExtractExplorerItem<'FilePathDir'>[], ExtractExplorerItem<'FilePathWithAssetObject'>[]]>(
      ([folders, medias], item) => {
        if (item.type === 'FilePathDir') {
          folders.push(item)
        } else {
          medias.push(item)
        }
        // FIXME don't know why ts throw error on this type
        return [folders, medias]
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
