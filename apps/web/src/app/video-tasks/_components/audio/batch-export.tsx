import TaskDropdownMenu from '../TaskDropdownMenu'
import Icon from '@gendam/ui/icons'
import MuseMultiSelect from '@/app/video-tasks/_components/MultiSelect'
import { toast } from 'sonner'
import { AudioType, ExportInput } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { cn } from '@/lib/utils'
import { Button } from '@gendam/ui/v2/button'
import { ScrollArea } from '@gendam/ui/v1/scroll-area'
import Image from 'next/image'
import { produce } from 'immer'
import { useCallback, useMemo, useState } from 'react'
import { useBoundStore } from '../../_store'
import { WithDownloadDialogButton } from '../withDownloadDialog'
import { FileTypeEnum } from './export'

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

  const moreActionOptions = useCallback(
    (id: string) => {
      return [
        {
          label: (
            <div className="flex items-center gap-1.5">
              <Icon.Cycle />
              <span>Apply export formats to all</span>
            </div>
          ),
          handleSelect: () => {
            const types = multiValues.find((v) => v.id === id)?.types || []
            data.filter((d) => d.id !== id).forEach((d) => updateItemTypes(d.id, types))
          },
        },
        {
          label: (
            <div className="flex items-center gap-1.5">
              <Icon.ArrowUpLeft />
              <span>Reset export options</span>
            </div>
          ),
          handleSelect: () => updateItemTypes(id, []),
        },
      ]
    },
    [data, multiValues],
  )

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
    <div className="flex flex-col">
      <div className="grid grid-cols-10 border-b border-app-line px-6 py-2 text-xs font-normal leading-4">
        <p className="col-span-5">File</p>
        <p className="col-span-3">Formats</p>
        <p className="col-span-1">Quantity</p>
        <div className="col-span-1"></div>
      </div>
      <ScrollArea className="h-[30rem]">
        {data.map(({ id, label, assetObjectHash }, index) => (
          <div
            key={id}
            className={cn(
              'grid grid-cols-10 items-center px-6 py-3 border-app-line',
              data.length === index + 1 ? 'border-b-0' : 'border-b',
            )}
          >
            <div className="col-span-5 flex items-center gap-4">
              <div className="relative h-9 w-9">
                {/* <video controls={false} autoPlay={false} muted loop style={{ width: '100%', height: '100%', objectFit: 'cover' }} className="h-9 w-9">
                  <source src={currentLibrary.getFileSrc(assetObjectHash)} />
                </video> */}
                <Image
                  src={currentLibrary.getThumbnailSrc(assetObjectHash)}
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
              <TaskDropdownMenu options={moreActionOptions(id)} />
            </div>
          </div>
        ))}
      </ScrollArea>
      <div className="flex flex-1 justify-end gap-2 border-t border-app-line px-6 py-2.5">
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
