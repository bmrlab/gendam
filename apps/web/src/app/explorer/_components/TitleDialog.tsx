import { useExplorerContext } from '@/Explorer/hooks'
import { useExplorerApiContext } from '@/Explorer/hooks/useExplorerApi'
import { queryClient, rspc } from '@/lib/rspc'
import { Button } from '@gendam/ui/v2/button'
import { Dialog } from '@gendam/ui/v2/dialog'
import { Form } from '@gendam/ui/v2/form'
import { useCallback, useState } from 'react'
import { create } from 'zustand'

interface TitleDialogState {
  open: boolean
  setOpen: (open: boolean) => void
}

export const useTitleDialog = create<TitleDialogState>((set) => ({
  open: false,
  setOpen: (open) => set({ open }),
}))

const TitleDialog: React.FC = () => {
  const explorerApi = useExplorerApiContext()
  const titleDialog = useTitleDialog()
  const explorer = useExplorerContext()
  const createDirMut = rspc.useMutation(['assets.create_dir'])
  const [title, setTitle] = useState('')

  const onSubmit = useCallback(
    async (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()
      if (!title || !explorer.materializedPath) {
        return
      }
      try {
        await createDirMut.mutateAsync({
          materializedPath: explorer.materializedPath,
          name: title,
        })
        titleDialog.setOpen(false)
        setTitle('')
      } catch (error) {}
      queryClient.invalidateQueries({
        queryKey: [explorerApi.listApi, { materializedPath: explorer.materializedPath }],
      })
    },
    [createDirMut, explorer.materializedPath, title, titleDialog],
  )

  return (
    <Dialog.Root
      open={titleDialog.open}
      onOpenChange={(open) => {
        titleDialog.setOpen(open)
        if (!open) setTitle('')
      }}
    >
      <Dialog.Portal>
        <Dialog.Overlay onClick={(e) => e.stopPropagation()} />
        <Dialog.Content onClick={(e) => e.stopPropagation()} className="w-96 px-4 pb-6 pt-4">
          <Form.Root onSubmit={onSubmit}>
            <Form.Field name="title" className="flex flex-col items-stretch justify-center gap-5">
              <Form.Label>Folder Name</Form.Label>
              <Form.Input size="md" value={title} onChange={(e) => setTitle(e.currentTarget.value)} />
              <Button type="submit" variant="accent" disabled={createDirMut.isPending}>
                Create
              </Button>
            </Form.Field>
          </Form.Root>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}

export default TitleDialog
