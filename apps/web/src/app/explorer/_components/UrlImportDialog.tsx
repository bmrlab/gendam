import { useExplorerContext } from '@/Explorer/hooks'
import { queryClient, rspc } from '@/lib/rspc'
import { Button } from '@gendam/ui/v2/button'
import { Dialog } from '@gendam/ui/v2/dialog'
import { Form } from '@gendam/ui/v2/form'
import { useCallback, useState } from 'react'
import { create } from 'zustand'

interface UrlImportDialogState {
  open: boolean
  setOpen: (open: boolean) => void
}

export const useUrlImportDialog = create<UrlImportDialogState>((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
}))

const UrlImportDialog: React.FC = () => {
  const urlImportDialog = useUrlImportDialog()
  const explorer = useExplorerContext()
  const uploadUrlMut = rspc.useMutation(['assets.create_web_page_object'])
  const [url, setUrl] = useState('')

  const onSubmit = useCallback(
    async (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()
      if (!url || !explorer.materializedPath) {
        return
      }
      try {
        await uploadUrlMut.mutateAsync({
          materializedPath: explorer.materializedPath,
          url: url,
        })
        urlImportDialog.setOpen(false)
        setUrl('')
      } catch (error) {}
      queryClient.invalidateQueries({
        queryKey: ['assets.list', { materializedPath: explorer.materializedPath }],
      })
    },
    [uploadUrlMut, explorer.materializedPath, url, urlImportDialog],
  )

  return (
    <Dialog.Root
      open={urlImportDialog.open}
      onOpenChange={(open) => {
        urlImportDialog.setOpen(open)
        if (!open) setUrl('')
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay onClick={(e) => e.stopPropagation()} />
        <Dialog.Content onClick={(e) => e.stopPropagation()} className="w-96 px-4 pb-6 pt-4">
          <Form.Root onSubmit={onSubmit}>
            <Form.Field name="url" className="flex flex-col items-stretch justify-center gap-5">
              <Form.Label>URL</Form.Label>
              <Form.Input size="md" value={url} onChange={(e) => setUrl(e.currentTarget.value)} />
              <Button type="submit" variant="accent" disabled={uploadUrlMut.isPending}>
                Import
              </Button>
            </Form.Field>
          </Form.Root>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}

export default UrlImportDialog
