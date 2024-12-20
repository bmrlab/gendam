import { AudioType, ExportInput } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { cn } from '@/lib/utils'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import { DropdownMenu } from '@gendam/ui/v2/dropdown-menu'
import { produce } from 'immer'
import Image from 'next/image'
import { useMemo, useState } from 'react'
import { toast } from 'sonner'
import { FileTypeEnum } from './AudioExport'
import { useBoundStore } from './store'
import MuseMultiSelect from './ui/MultiSelect'
import { WithDownloadDialogButton } from './withDownloadDialog'

export type BatchExportProps = {
  id: string
  label: string
  assetObjectId: number
  assetObjectHash: string
}[]

export default function BatchExport() {
  const currentLibrary = useCurrentLibrary()

  const audioDialogProps = useBoundStore.use.audioDialogProps()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()

  const { mutateAsync: batchExport } = rspc.useMutation('audio.batch_export')

  const data = useMemo(() => audioDialogProps.params as BatchExportProps, [audioDialogProps.params])

  const [multiValues, setMultiValues] = useState<{ id: string; types: string[] }[]>(
    data.map((option) => ({ id: option.id, types: [] })),
  )

  const updateItemTypes = (id: string, newTypes: string[]) => {
    setMultiValues((currentValues) =>
      produce(currentValues, (draft) => {
        const item = draft.find((item) => item.id === id)
        if (item) {
          item.types = newTypes
        }
      }),
    )
  }

  const handleExport = async (dir: string) => {
    const input: ExportInput[] = multiValues.map(({ id, types }) => ({
      hash: id,
      types: types as AudioType[],
      path: dir,
      fileName: data.find((d) => d.id === id)?.label,
    }))
    const errorList = await batchExport(input)
    setIsOpenAudioDialog(false)
    if (errorList.length > 0) {
      toast.error(`${errorList.join(', ')}, export failed`)
    } else {
      toast.success('Export successfully')
    }
  }

  return (
    <div className="flex flex-1 flex-col overflow-hidden">
      <div className="border-app-line grid grid-cols-10 border-b px-6 py-2 text-xs font-normal leading-4">
        <p className="col-span-5">File</p>
        <p className="col-span-3">Formats</p>
        <p className="col-span-1">Quantity</p>
        <div className="col-span-1"></div>
      </div>
      <div className="flex-1 overflow-auto pb-16">
        {data.map(({ id, label, assetObjectHash }, index) => (
          <div
            key={id}
            className={cn(
              'border-app-line grid grid-cols-10 items-center px-6 py-3',
              data.length === index + 1 ? 'border-b-0' : 'border-b',
            )}
          >
            <div className="col-span-5 flex items-center gap-4">
              <div className="relative h-9 w-9">
                {/* <video controls={false} autoPlay={false} muted loop style={{ width: '100%', height: '100%', objectFit: 'cover' }} className="h-9 w-9">
                  <source src={currentLibrary.getFileSrc(assetObjectHash)} />
                </video> */}
                <Image
                  src={currentLibrary.getThumbnailSrc(assetObjectHash, 'Video')}
                  alt={assetObjectHash}
                  fill={true}
                  className="object-cover"
                  priority
                ></Image>
              </div>
              <p className="truncate text-xs font-medium leading-4">{label}</p>
            </div>
            <div className="col-span-3 max-w-48">
              <MuseMultiSelect
                value={multiValues.find((v) => v.id === id)?.types || []}
                onValueChange={(value) => updateItemTypes(id, value)}
                showValue
                placeholder="Select formats"
                options={Object.keys(FileTypeEnum).map((type) => ({
                  label: FileTypeEnum[type as keyof typeof FileTypeEnum],
                  value: type,
                }))}
              />
            </div>
            <div className="col-span-1 text-sm leading-6">
              {(multiValues.find((v) => v.id === id)?.types || []).length}
            </div>
            <div className="col-span-1 cursor-pointer">
              <DropdownMenu.Root>
                <DropdownMenu.Trigger asChild>
                  <div className="border-app-line data-[state=open]:bg-app-hover inline-flex size-6 cursor-default items-center justify-center rounded border">
                    <span className="sr-only">Open menu</span>
                    <Icon.MoreVertical />
                  </div>
                </DropdownMenu.Trigger>
                <DropdownMenu.Content align="end">
                  <DropdownMenu.Item
                    onSelect={() => {
                      const types = multiValues.find((v) => v.id === id)?.types || []
                      data.filter((d) => d.id !== id).forEach((d) => updateItemTypes(d.id, types))
                    }}
                  >
                    <div className="flex items-center gap-1.5">
                      <Icon.Cycle />
                      <span>Apply export formats to all</span>
                    </div>
                  </DropdownMenu.Item>
                  <DropdownMenu.Item onSelect={() => updateItemTypes(id, [])}>
                    <div className="flex items-center gap-1.5">
                      <Icon.ArrowUpLeft />
                      <span>Reset export options</span>
                    </div>
                  </DropdownMenu.Item>
                </DropdownMenu.Content>
              </DropdownMenu.Root>
            </div>
          </div>
        ))}
      </div>
      <div className="border-app-line flex justify-end gap-2 border-t px-6 py-2.5">
        <Button variant="outline" size="md" onClick={() => setIsOpenAudioDialog(false)}>
          Cancel
        </Button>
        <WithDownloadDialogButton variant="accent" size="md" onSelection={handleExport}>
          Export
        </WithDownloadDialogButton>
      </div>
    </div>
  )
}
