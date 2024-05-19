import { type FilePath } from '@/lib/bindings'
import { useFoldersDialog } from '@/components/Shared/FoldersDialog/store'

export const useOpenFileSelection = () => {
  const foldersDialog = useFoldersDialog()
  function openFileSelection(): Promise<FilePath | null> {
    foldersDialog.setOpen(true)

    return new Promise((resolve, reject) => {
      const handleConfirm = (path: FilePath | null) => {
        resolve(path)
      }
      foldersDialog.setConfirm(handleConfirm)
    })
  }

  return {
    openFileSelection,
  }
}
