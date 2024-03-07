import { FileTypeEnum } from '@/app/video-tasks/_compoents/audio/export'
import MuseDropdownMenu from '@/components/DropdownMenu'
import Icon from '@/components/Icon'
import MuseMultiSelect from '@/components/MultiSelect'
import { ScrollArea } from '@/components/ui/scroll-area'
import Image from 'next/image'
import { useMemo } from 'react'

export default function BatchExport() {
  const moreActionOptions = useMemo(() => {
    return [
      {
        label: (
          <div className="flex items-center gap-1.5">
            <Icon.regenerate />
            <span>把导出格式应用到全部</span>
          </div>
        ),
        handleClick: () => {},
      },
      {
        label: (
          <div className="flex items-center gap-1.5">
            <Icon.arrowUpLeft />
            <span>重设导出格式</span>
          </div>
        ),
        handleClick: () => {},
      },
    ]
  }, [])

  return (
    <div>
      <div className="grid grid-cols-10 border-b px-6 py-2 text-[11px] font-normal leading-[14px] text-black/80">
        <p className="col-span-5">文件</p>
        <p className="col-span-3">格式</p>
        <p className="col-span-1">数量</p>
        <div className="col-span-1"></div>
      </div>
      <ScrollArea className="h-[200px]">
        <div className="grid grid-cols-10 items-center border-b px-6 py-3">
          <div className="col-span-5 flex items-center gap-[30px]">
            <div className="relative h-9 w-9 bg-[#F6F7F9]">
              <Image src="https://placehold.co/100x200" alt="" fill className="object-contain" />
            </div>
            <p className="text-[13px] font-medium leading-[18px] text-[#323438]">MUSE的视频</p>
          </div>
          <div className="col-span-3 w-[240px]">
            <MuseMultiSelect
              showValue
              placeholder="选择格式"
              options={Object.keys(FileTypeEnum).map((type) => ({
                label: FileTypeEnum[type as keyof typeof FileTypeEnum],
                value: type.toString(),
              }))}
            />
          </div>
          <div className="col-span-1 text-[13px] font-medium leading-[18px] text-[#262626]">1</div>
          <div className="col-span-1 flex size-[25px] cursor-pointer items-center justify-center rounded text-[#676C77] hover:bg-[#EBECEE]">
            <MuseDropdownMenu options={moreActionOptions} />
          </div>
        </div>
      </ScrollArea>
    </div>
  )
}
