'use client'
import PageNav from '@/components/PageNav'
import Viewport from '@/components/Viewport'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import { Checkbox } from '@gendam/ui/v2/checkbox'
import { Form } from '@gendam/ui/v2/form'
import { useCallback, useEffect, useState } from 'react'
import { toast } from 'sonner'
import S3Config from './s3_settings'

const AccountDetail: React.FC = () => {
  const { data: auth } = rspc.useQuery(['users.get'])

  return (
    <div>
      <div className="mb-4 text-xl font-medium">Account</div>
      <div className="text-ink/75 bg-app-box border-app-line mb-8 flex h-64 w-64 flex-col items-center justify-center gap-4 rounded-xl border">
        <div className="bg-app-overlay h-24 w-24 rounded-full p-6">
          <Icon.Person className="h-full w-full" />
        </div>
        <div className="text-center text-sm">
          Welcome
          <br />
          {auth?.name}
        </div>
      </div>
    </div>
  )
}

const LibrarySettings: React.FC = () => {
  const currentLibrary = useCurrentLibrary()
  const [title, setTitle] = useState('')
  const [alwaysDeleteAfterUploadChecked, setAlwaysDeleteAfterUploadChecked] = useState(false)
  const [isPending, setIsPending] = useState(false)

  const onSubmit = useCallback(
    async (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()
      setIsPending(true)
      try {
        await currentLibrary.updateLibrarySettings({ title })
        toast.success('Library settings updated')
      } catch (error) {
        console.error(error)
      }
      setIsPending(false)
      // setTimeout(() => {
      //   window.location.reload()
      // }, 500)
    },
    [currentLibrary, title],
  )

  const handleAlwaysDeleteLocalFileAfterUpload = useCallback(
    async (flag: boolean) => {
      try {
        await currentLibrary.updateLibrarySettings({ alwaysDeleteLocalFileAfterUpload: flag })
        toast.success('Library settings updated')
      } catch (error) {
        console.error(error)
      }
    },
    [currentLibrary],
  )

  useEffect(() => {
    setTitle(currentLibrary.librarySettings.title)
    setAlwaysDeleteAfterUploadChecked(currentLibrary.librarySettings.alwaysDeleteLocalFileAfterUpload)
  }, [currentLibrary.librarySettings.title])

  return (
    <div>
      <div className="mb-8 text-xl font-medium">Library Settings</div>
      <Form.Root onSubmit={onSubmit} className="mb-8">
        <Form.Field name="title" className="flex items-center justify-start gap-3">
          <Form.Label>Title</Form.Label>
          <Form.Input size="md" value={title} onChange={(e) => setTitle(e.currentTarget.value)} />
          <Button type="submit" variant="accent" disabled={isPending}>
            Save
          </Button>
        </Form.Field>
      </Form.Root>
      <div className="text-ink/75 flex items-center space-x-2">
        <Checkbox.Root
          id="always-delete-checkbox"
          checked={alwaysDeleteAfterUploadChecked}
          onCheckedChange={(e) => {
            setAlwaysDeleteAfterUploadChecked(e as boolean)
            handleAlwaysDeleteLocalFileAfterUpload(e as boolean)
          }}
        >
          <Checkbox.Indicator />
        </Checkbox.Root>
        <label htmlFor="always-delete-checkbox">Always delete local file after upload</label>
      </div>
    </div>
  )
}

export default function Settings() {
  return (
    <Viewport.Page>
      <Viewport.Toolbar>
        <PageNav title="Settings" />
      </Viewport.Toolbar>
      <Viewport.Content className="p-6">
        <AccountDetail />
        <div className="bg-app-line my-4 h-px"></div>
        <LibrarySettings />
        <div className="bg-app-line my-4 h-px"></div>
        <S3Config />
      </Viewport.Content>
    </Viewport.Page>
  )
}
