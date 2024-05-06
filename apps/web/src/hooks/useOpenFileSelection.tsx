import { ExplorerItem } from '@/Explorer/types'
import { useFoldersDialog } from '@/app/explorer/_components/FoldersDialog'

export const useOpenFileSelection = () => {
  const foldersDialog = useFoldersDialog()
  function openFileSelection(): Promise<ExplorerItem | null> {
    foldersDialog.setOpen(true)

    return new Promise((resolve, reject) => {
      const handleConfirm = (path: ExplorerItem | null) => {
        resolve(path)
      }
      foldersDialog.setConfirm(handleConfirm)
    })
  }

  return {
    openFileSelection,
  }
}
