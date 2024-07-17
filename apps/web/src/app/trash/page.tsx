'use client'
import { ExplorerViewContextProvider } from '@/Explorer/hooks'
import { ExplorerApiContextProvider } from '@/Explorer/hooks/useExplorerApi'
import { CommonPage } from '../explorer/_components'
import ItemContextMenu from './_components/ItemContextMenu'

export default function TrashPage() {
  return (
    <ExplorerApiContextProvider
      value={{
        listApi: 'assets.trash',
        moveApi: 'assets.move_trash_file_path',
      }}
    >
      <ExplorerViewContextProvider
        value={{
          contextMenu: (data) => (data.type === 'FilePath' ? <ItemContextMenu data={data.filePath} /> : null),
        }}
      >
        <CommonPage />
      </ExplorerViewContextProvider>
    </ExplorerApiContextProvider>
  )
}
