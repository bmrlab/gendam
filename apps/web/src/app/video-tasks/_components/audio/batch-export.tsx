import MuseDropdownMenu from '@/components/DropdownMenu'
import Icon from '@/components/Icon'
import MuseMultiSelect from '@/components/MultiSelect'
import { useToast } from '@/components/Toast/use-toast'
import { AudioType, ExportInput } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { cn } from '@/lib/utils'
import { Button } from '@muse/ui/v1/button'
import { ScrollArea } from '@muse/ui/v1/scroll-area'
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
  const { toast } = useToast()

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
              <Icon.regenerate />
              <span>把导出格式应用到全部</span>
            </div>
          ),
          handleClick: () => {
            const types = multiValues.find((v) => v.id === id)?.types || []
            data.filter((d) => d.id !== id).forEach((d) => updateItemTypes(d.id, types))
          },
        },
        {
          label: (
            <div className="flex items-center gap-1.5">
              <Icon.arrowUpLeft />
              <span>重设导出格式</span>
            </div>
          ),
          handleClick: () => updateItemTypes(id, []),
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
      toast({
        title: `${errorList.join('、')}，格式导出失败`,
        variant: 'destructive',
      })
    } else {
      toast({
        title: '导出成功',
      })
    }
  }

  return (
    <div className="flex flex-col">
      <div className="grid grid-cols-10 border-b px-6 py-2 text-[11px] font-normal leading-[14px] text-black/80">
        <p className="col-span-5">文件</p>
        <p className="col-span-3">格式</p>
        <p className="col-span-1">数量</p>
        <div className="col-span-1"></div>
      </div>
      <ScrollArea className="h-[576px]">
        {data.map(({ id, label, assetObjectHash }, index) => (
          <div
            key={id}
            className={cn(
              'grid grid-cols-10 items-center px-6 py-3',
              data.length === index + 1 ? 'border-b-0' : 'border-b',
            )}
          >
            <div className="col-span-5 flex items-center gap-[30px]">
              <div className="relative h-9 w-9 bg-[#F6F7F9]">
                {/*<Image src={image} alt="" fill className="object-contain" />*/}
                <video
                  controls={false}
                  autoPlay={false}
                  muted
                  loop
                  style={{
                    width: '100%',
                    height: '100%',
                    objectFit: 'cover',
                  }}
                  className="h-9 w-9"
                >
                  <source src={currentLibrary.getFileSrc(assetObjectHash)} />
                </video>
              </div>
              <p className="truncate text-[13px] font-medium leading-[18px] text-[#323438]">{label}</p>
            </div>
            <div className="col-span-3 max-w-[240px]">
              <MuseMultiSelect
                value={multiValues.find((v) => v.id === id)?.types || []}
                onValueChange={(value) => updateItemTypes(id, value)}
                showValue
                placeholder="选择格式"
                options={Object.keys(FileTypeEnum).map((type) => ({
                  label: FileTypeEnum[type as keyof typeof FileTypeEnum],
                  value: type,
                }))}
              />
            </div>
            <div className="col-span-1 text-[13px] font-medium leading-[18px] text-[#262626]">
              {(multiValues.find((v) => v.id === id)?.types || []).length}
            </div>
            <div className="col-span-1 flex size-[25px] cursor-pointer items-center justify-center rounded text-[#676C77] hover:bg-[#EBECEE]">
              <MuseDropdownMenu options={moreActionOptions(id)} />
            </div>
          </div>
        ))}
      </ScrollArea>
      <div className="flex flex-1 justify-end gap-2 border-t border-[#EBECEE] px-6 py-2.5">
        <Button variant="outline" className="px-[41px]" onClick={() => setIsOpenAudioDialog(false)}>
          取消
        </Button>
        <WithDownloadDialogButton className="px-[76px]" onSelection={handleExport}>
          导出
        </WithDownloadDialogButton>
      </div>
    </div>
  )
}
