import { S3Config } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { Button } from '@gendam/ui/v2/button'
import { Form, FormPrimitive } from '@gendam/ui/v2/form'
import { useCallback, useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import { toast } from 'sonner'

export default function S3Config() {
  const { t } = useTranslation()
  const currentLibrary = useCurrentLibrary()

  const [config, setConfig] = useState<S3Config>({
    bucket: '',
    endpoint: '',
    accessKeyId: '',
    secretAccessKey: '',
  })

  const [isPending, setIsPending] = useState(false)

  useEffect(() => {
    if (!currentLibrary.librarySettings.s3Config) return
    setConfig(currentLibrary.librarySettings.s3Config)
  }, [currentLibrary.librarySettings.s3Config])

  const onSubmit = useCallback(
    async (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()
      setIsPending(true)
      try {
        await currentLibrary.updateLibrarySettings({ s3Config: config })
        toast.success(t('setting.s3.submit.success'))
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
      <div className="mb-8 text-xl font-medium">{t('setting.s3.title')}</div>
      <Form.Root onSubmit={onSubmit} className="mb-8 max-w-md space-y-4">
        <Form.Field name="bucket" className="grid">
          <Form.Label>{t('setting.s3.bucket')}</Form.Label>
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
            {t('setting.s3.bucket.placeholder')}
          </FormPrimitive.Message>
        </Form.Field>
        <Form.Field name="endpoint" className="grid">
          <Form.Label>{t('setting.s3.endpoint')}</Form.Label>
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
            {t('setting.s3.endpoint.placeholder')}
          </FormPrimitive.Message>
        </Form.Field>
        <Form.Field name="accessKeyId" className="grid">
          <Form.Label>{t('setting.s3.accessKeyId')}</Form.Label>
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
            {t('setting.s3.accessKeyId.placeholder')}
          </FormPrimitive.Message>
        </Form.Field>
        <Form.Field name="secretAccessKey" className="grid">
          <Form.Label>{t('setting.s3.secretAccessKey')}</Form.Label>
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
            {t('setting.s3.secretAccessKey.placeholder')}
          </FormPrimitive.Message>
        </Form.Field>
        <FormPrimitive.Submit asChild>
          <Button type="submit" variant="accent" disabled={isPending}>
            {t('setting.s3.save')}
          </Button>
        </FormPrimitive.Submit>
      </Form.Root>
    </div>
  )
}
