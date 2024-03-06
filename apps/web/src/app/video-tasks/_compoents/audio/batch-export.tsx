import { FileTypeEnum } from '@/app/video-tasks/_compoents/audio/export'
import MuseMultiSelect from '@/components/MultiSelect'
import { ScrollArea } from '@/components/ui/scroll-area'
import Image from 'next/image'

export default function BatchExport() {
  return (
    <div>
      <div className="grid grid-cols-4 border-b px-6 py-2 text-[11px] font-normal leading-[14px] text-black/80">
        <p>文件</p>
        <p>格式</p>
        <p>数量</p>
        <></>
      </div>
      <ScrollArea className="h-[200px]">
        <div className="grid grid-cols-4 border-b px-6 py-3">
          <div className="col-span-2 flex items-center gap-[30px]">
            <div className="relative h-9 w-9 bg-[#F6F7F9]">
              <Image src="https://placehold.co/100x200" alt="" fill className="object-contain" />
            </div>
            <p className="text-[13px] font-medium leading-[18px] text-[#323438]">MUSE的视频</p>
          </div>
          <div className="col-span-1 w-[240px]">
            <MuseMultiSelect
              showValue
              placeholder="选择格式"
              options={Object.keys(FileTypeEnum).map((type) => ({
                label: FileTypeEnum[type as keyof typeof FileTypeEnum],
                value: type.toString(),
              }))}
            />
          </div>
          <div className="col-span-1">数量</div>
          <></>
        </div>
      </ScrollArea>
    </div>
  )
}
