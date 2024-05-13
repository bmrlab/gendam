import { SingleExportProps } from '../../_store/audio-dialog'
import { RoundedBadge } from '../Badge'
import MuseRadio from '../Radio'
import Icon from '@gendam/ui/icons'
import { ScrollArea } from '@gendam/ui/v1/scroll-area'
import { toast } from 'sonner'
import { WithDownloadDialogButton } from '../withDownloadDialog'
import { AudioType } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { cn } from '@/lib/utils'
import { useBoundStore } from '../../_store'
import { Fragment, useMemo, useState } from 'react'

export enum FileTypeEnum {
  txt = 'Plain Text (.txt)',
  srt = 'SubRip (.srt)',
  vtt = 'WebVTT (.vtt)',
  json = 'JSON',
  csv = 'CSV',
  docx = 'Word Document (.docx)',
  ale = 'Avid Log Exchange (.ale)',
}

export default function AudioExport() {
  const audioDialogProps = useBoundStore.use.audioDialogProps()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()
  const fileHash = useMemo(() => (audioDialogProps.params as SingleExportProps)?.fileHash, [audioDialogProps])

  const { data: rawData, isLoading, error } = rspc.useQuery(['audio.find_by_hash', fileHash as string])
  const { mutateAsync } = rspc.useMutation(['audio.export'])

  const data = useMemo(() => {
    return Object.keys(FileTypeEnum).map((key) => {
      return {
        type: FileTypeEnum[key as keyof typeof FileTypeEnum],
        content: rawData?.find((item) => item.type === key)?.content ?? '',
      }
    })
  }, [rawData])

  // 顶部的文件类型
  const [fileType, setFileType] = useState(FileTypeEnum.txt)

  const currentContent = useMemo(() => data?.find((item) => item.type === fileType)?.content, [data, fileType])

  const [selectFileGroup, setSelectFileGroup] = useState<boolean[]>([
    true,
    ...new Array(Object.keys(FileTypeEnum).length - 1).fill(false),
  ])

  const toggleSelectGroup = (index: number) => {
    // 至少选择一项
    if (selectFileGroup.filter((s) => s).length === 1 && selectFileGroup[index]) return

    if (!selectFileGroup[index]) {
      setFileType(Object.values(FileTypeEnum)[index])
    }

    const updatedBools = selectFileGroup.map((item, i) => (i === index ? !item : item))
    setSelectFileGroup(updatedBools)
  }

  const handleDownload = async (dir: string) => {
    let types: AudioType[] = []
    selectFileGroup.forEach((item, index) => {
      if (item) {
        types.push(Object.keys(FileTypeEnum)[index] as AudioType)
      }
    })
    const errorList = await mutateAsync({
      types: types,
      hash: fileHash as string,
      path: dir,
    })
    if (errorList.length > 0) {
      toast.error(`${errorList.join(', ')}, export failed`)
    } else {
      toast.success('Export successfully')
    }
    setIsOpenAudioDialog(false)
  }

  return (
    <div className="flex-1 flex items-stretch overflow-hidden">
      <div className="flex-1 flex flex-col border-r border-app-line overflow-hidden">
        <div className="flex gap-2 flex-wrap content-start items-start justify-start w-full py-4 px-6">
          {Object.entries(FileTypeEnum).map(([key, value]) => {
            return (
              <label
                key={key}
                className={cn(
                  'select-none rounded px-3 py-2 text-xs font-medium leading-5 transition hover:cursor-pointer ',
                  fileType === value ? 'bg-app-hover shadow-xs' : 'hover:bg-app-hover',
                )}
                onClick={() => setFileType(value)}
              >
                {value}
              </label>
            )
          })}
        </div>
        <div className="flex-1 relative px-6 pb-6 overflow-hidden">
          <ScrollArea className="h-full w-full rounded-md bg-app-overlay border border-app-line">
            <p className="p-4 text-sm font-normal leading-8">
              {(currentContent || '').split('\n').map((line: string, index: number) => (
                <Fragment key={index}>
                  {line}
                  <br />
                </Fragment>
              ))}
            </p>
          </ScrollArea>
          <RoundedBadge
            className="absolute bottom-8 right-8"
            onClick={() => navigator.clipboard.writeText(currentContent ?? '')}
          >
            <div className="flex gap-0.5">
              <Icon.CopySimple />
              <span className="select-none">Copy</span>
            </div>
          </RoundedBadge>
        </div>
      </div>
      <div className="h-auto overflow-scroll w-60 px-6 pb-6 pt-4">
        <div className="flex flex-col gap-3">
          <p className="text-ink/50 text-xs font-medium leading-3">FILE FORMATS</p>
          {
            // @ts-ignore
            Object.entries(FileTypeEnum).map(([key, value], index) => {
              return (
                <MuseRadio
                  key={key}
                  label={value}
                  active={selectFileGroup[index]}
                  onClick={() => toggleSelectGroup(index)}
                />
              )
            })
          }
        </div>
        <div className="mt-5 flex flex-col gap-3">
          <p className="text-ink/50 text-xs font-medium leading-3">EXPORT OPTIONS</p>
          <MuseRadio label="Show Timecodes" />
          <MuseRadio label="Show Speakers" />
        </div>
        <div className="flex w-full flex-1 items-end">
          <WithDownloadDialogButton className="mt-4 w-full" onSelection={handleDownload}>
            Export
          </WithDownloadDialogButton>
        </div>
      </div>
    </div>
  )
}
