'use client'
import { queryClient, rspc } from '@/lib/rspc'
import { GenDAM_Logo } from '@muse/assets/images'
import Icon from '@muse/ui/icons'
import Image from 'next/image'
import { useCallback, useState } from 'react'
import { Dialog } from '@muse/ui/v2/dialog'
import { Form } from '@muse/ui/v2/form'
import { Button } from '@muse/ui/v2/button'

export default function LibrariesSelect({
  switchCurrentLibraryById,
}: {
  switchCurrentLibraryById: (libraryId: string) => Promise<void>
}) {
  const [dialogOpen, setDialogOpen] = useState(false)
  const [title, setTitle] = useState('')
  const librariesQuery = rspc.useQuery(['libraries.list'])
  const libraryCreateMut = rspc.useMutation('libraries.create')

  const onSubmit = useCallback(
    async (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()
      try {
        await libraryCreateMut.mutateAsync(title)
        setDialogOpen(false)
        setTitle('')
        queryClient.invalidateQueries({
          queryKey: ['libraries.list'],
        })
      } catch(error: any) {}
    },
    [libraryCreateMut, title],
  )

  return (
    <div className="bg-app flex h-screen w-screen flex-col items-center justify-center">
      <Image src={GenDAM_Logo} alt="Muse" className="mb-4 h-8 w-8"></Image>
      {librariesQuery.isSuccess ? (
        <div className="border-app-line bg-app-box my-4 w-80 rounded-md border p-1 shadow-sm">
          {librariesQuery.data.length === 0 ? (
            <div className="text-ink/60 px-3 py-2 text-center text-xs">
              No library has been created yet, continue by clicking &quot;Create&quot; below.
            </div>
          ) : (
            <div className="text-ink/60 px-3 py-2 text-center text-xs">Select Library</div>
          )}
          {librariesQuery.data.map((library, index: number) => {
            return (
              <div
                key={library.id}
                className="hover:bg-app-hover flex items-center justify-start rounded-md px-3 py-2"
                onClick={() => switchCurrentLibraryById(library.id)}
              >
                <Image src={GenDAM_Logo} alt="Muse" className="h-8 w-8"></Image>
                <div className="mx-2 w-64 truncate text-xs font-semibold">
                  {library.title ?? 'Untitled'} ({library.id})
                </div>
              </div>
            )
          })}
          <div className="hover:bg-app-hover rounded-md px-3 py-2" onClick={() => setDialogOpen(true)}>
            <div className="text-center text-sm">Create a library</div>
          </div>
        </div>
      ) : (
        <div className="text-ink/50 my-8 text-center text-sm">
          <Icon.Loading className="inline-block h-8 w-8 animate-spin" />
          <div className="mt-8">Loading library</div>
        </div>
      )}
      <Dialog.Root open={dialogOpen} onOpenChange={(open) => {
        setDialogOpen(open)
        if (!open) setTitle('')
      }}>
        <Dialog.Portal>
          <Dialog.Overlay onClick={(e) => e.stopPropagation()} />
          <Dialog.Content onClick={(e) => e.stopPropagation()} className="w-96 px-4 pb-6 pt-4">
            <Form.Root onSubmit={onSubmit}>
              <Form.Field name="title" className="flex flex-col items-stretch justify-center gap-5">
                <Form.Label>Library name</Form.Label>
                <Form.Input size="md" value={title} onChange={(e) => setTitle(e.currentTarget.value)} />
                <Button type="submit" variant="accent" disabled={libraryCreateMut.isPending}>
                  Create
                </Button>
              </Form.Field>
            </Form.Root>
          </Dialog.Content>
        </Dialog.Portal>
      </Dialog.Root>
    </div>
  )
}
