import { S3Config } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { Button } from '@gendam/ui/v2/button'
import { Form, FormPrimitive } from '@gendam/ui/v2/form'
import { useCallback, useState } from 'react'
import { toast } from 'sonner'

export default function S3Config() {
  const currentLibrary = useCurrentLibrary()

  const [config, setConfig] = useState<S3Config>({
    bucket: '',
    endpoint: '',
    accessKeyId: '',
    secretAccessKey: '',
  })

  const [isPending, setIsPending] = useState(false)

  const onSubmit = useCallback(
    async (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()
      setIsPending(true)
      try {
        await currentLibrary.updateLibrarySettings({ s3Config: config })
        toast.success('Library settings updated')
      } catch (error) {
        console.error(error)
      }
      setIsPending(false)
      // setTimeout(() => {
      //   window.location.reload()
      // }, 500)
    },
    [currentLibrary, config],
  )

  return (
    <div>
      <div className="mb-8 text-xl font-medium">S3 Settings</div>
      <Form.Root onSubmit={onSubmit} className="mb-8 max-w-md space-y-4">
        <Form.Field name="bucket" className="grid">
          <Form.Label>Bucket</Form.Label>
          <Form.Input
            required
            size="md"
            value={config.bucket}
            onChange={(e) =>
              setConfig({
                ...config,
                bucket: e.currentTarget.value,
              })
            }
          />
          <FormPrimitive.Message className="text-[13px] text-red-500 opacity-[0.8]" match="valueMissing">
            Please enter your bucket
          </FormPrimitive.Message>
        </Form.Field>
        <Form.Field name="endpoint" className="grid">
          <Form.Label>Endpoint</Form.Label>
          <Form.Input
            required
            size="md"
            value={config.endpoint}
            onChange={(e) =>
              setConfig({
                ...config,
                endpoint: e.currentTarget.value,
              })
            }
          />
          <FormPrimitive.Message className="text-[13px] text-red-500 opacity-[0.8]" match="valueMissing">
            Please enter your endpoint
          </FormPrimitive.Message>
        </Form.Field>
        <Form.Field name="accessKeyId" className="grid">
          <Form.Label>AccessKeyId</Form.Label>
          <Form.Input
            required
            size="md"
            value={config.accessKeyId}
            onChange={(e) =>
              setConfig({
                ...config,
                accessKeyId: e.currentTarget.value,
              })
            }
          />
          <FormPrimitive.Message className="text-[13px] text-red-500 opacity-[0.8]" match="valueMissing">
            Please enter your access key id
          </FormPrimitive.Message>
        </Form.Field>
        <Form.Field name="secretAccessKey" className="grid">
          <Form.Label>SecretAccessKey</Form.Label>
          <Form.Input
            required
            size="md"
            value={config.secretAccessKey}
            onChange={(e) =>
              setConfig({
                ...config,
                secretAccessKey: e.currentTarget.value,
              })
            }
          />
          <FormPrimitive.Message className="text-[13px] text-red-500 opacity-[0.8]" match="valueMissing">
            Please enter your secret access key
          </FormPrimitive.Message>
        </Form.Field>
        <FormPrimitive.Submit asChild>
          <Button type="submit" variant="accent" disabled={isPending}>
            Save
          </Button>
        </FormPrimitive.Submit>
      </Form.Root>
    </div>
  )
}
