import MuseBadge from '@/components/Badge'
import Icon from '@/components/Icon'
import MuseRadio from '@/components/Radio'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { useToast } from '@/components/ui/use-toast'
import { AudioType } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { cn } from '@/lib/utils'
import { useBoundStore } from '@/store'
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
  const { toast } = useToast()

  const { fileHash } = useBoundStore.use.audioDialogProps()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()
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

  const [selectFileGroup, setSelectFileGroup] = useState<boolean[]>(
    new Array(Object.keys(FileTypeEnum).length).fill(false),
  )

  const toggleSelectGroup = (index: number) => {
    if (!selectFileGroup[index]) {
      setFileType(Object.values(FileTypeEnum)[index])
    }

    const updatedBools = selectFileGroup.map((item, i) => (i === index ? !item : item))
    setSelectFileGroup(updatedBools)
  }

  const handleDownload = async () => {
    let types: AudioType[] = []
    selectFileGroup.forEach((item, index) => {
      if (item) {
        types.push(Object.keys(FileTypeEnum)[index] as AudioType)
      }
    })
    const errorList = await mutateAsync({
      types: types,
      hash: fileHash as string,
      path: '/Users/zingerbee/Downloads',
    })
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
    setIsOpenAudioDialog(false)
  }

  return (
    <div className="flex size-full items-center">
      <div className="grid flex-1 border-r">
        <div className="w-[640px] py-4 pl-6">
          <div className="no-scrollbar flex w-full flex-nowrap items-center overflow-x-scroll whitespace-nowrap rounded bg-black/5 px-0.5 py-0.5 text-neutral-800">
            {Object.entries(FileTypeEnum).map(([key, value]) => {
              return (
                <label
                  key={key}
                  className={cn(
                    'select-none rounded px-2.5 py-[5px] text-[12px] font-medium leading-[14px] transition hover:cursor-pointer ',
                    fileType === value ? 'bg-white shadow-xs' : 'hover:bg-[#DDDDDE]',
                  )}
                  onClick={() => setFileType(value)}
                >
                  {value}
                </label>
              )
            })}
          </div>
        </div>
        <div className="relative p-6 pt-0">
          <ScrollArea className="h-[582px] w-[592px] rounded-[6px] border">
            <p className="p-4 text-[14px] font-normal leading-[21px] text-[#262626]">
              {(currentContent || '').split('\n').map((line: string, index: number) => (
                <Fragment key={index}>
                  {line}
                  <br />
                </Fragment>
              ))}
            </p>
          </ScrollArea>
          <MuseBadge
            className="absolute bottom-[34px] right-[34px]"
            onClick={() => navigator.clipboard.writeText(currentContent ?? '')}
          >
            <div className="flex gap-0.5">
              <Icon.copy />
              <span className="select-none">复制</span>
            </div>
          </MuseBadge>
        </div>
      </div>
      <div className="flex h-full w-[280px] flex-col justify-start px-6 pb-6 pt-4">
        <div className="flex flex-col gap-3">
          <p className="text-[14px] font-medium leading-5">文件格式</p>
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
        <div className="mt-2.5 flex flex-col gap-3">
          <p className="text-[14px] font-medium leading-5">导出选项</p>
          <MuseRadio label="11" />
          <MuseRadio label="222" />
        </div>
        <div className="flex w-full flex-1 items-end">
          <Button className="mt-4 w-full" onClick={handleDownload}>
            导出
          </Button>
        </div>
      </div>
    </div>
  )
}
