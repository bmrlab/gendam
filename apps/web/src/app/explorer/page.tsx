'use client'
import { ExplorerViewContextProvider } from '@/Explorer/hooks'
import { ExplorerApiContextProvider } from '@/Explorer/hooks/useExplorerApi'
import { CommonPage } from './_components'
import { FolderAdd } from './_components/FolderAdd'
import ItemContextMenu from './_components/ItemContextMenu'
import { UploadBtn } from './_components/UploadButton'

export default function ExplorerPage() {
  return (
    <ExplorerApiContextProvider
      value={{
        listApi: 'assets.list',
        moveApi: 'assets.move_file_path',
      }}
    >
      <ExplorerViewContextProvider
        value={{
          contextMenu: (data) => (data.type === 'FilePath' ? <ItemContextMenu data={data.filePath} /> : null),
          headerTools: (
            <>
              <FolderAdd />
              <UploadBtn />
            </>
          ),
        }}
      >
        <CommonPage />
      </ExplorerViewContextProvider>
    </ExplorerApiContextProvider>
  )
}
