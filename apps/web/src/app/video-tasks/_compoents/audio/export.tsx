import MuseBadge from '@/components/Badge'
import Icon from '@/components/Icon'
import MuseRadio from '@/components/Radio'
import { Button } from '@/components/ui/button'
import { ScrollArea } from '@/components/ui/scroll-area'
import { cn } from '@/lib/utils'
import { useState } from 'react'

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
  // 顶部的文件类型
  const [fileType, setFileType] = useState(FileTypeEnum.txt)

  const [selectFileGroup, setSelectFileGroup] = useState<boolean[]>(
    new Array(Object.keys(FileTypeEnum).length).fill(false),
  )

  const toggleSelectGroup = (index: number) => {
    const updatedBools = selectFileGroup.map((item, i) => (i === index ? !item : item))
    setSelectFileGroup(updatedBools)
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
          <ScrollArea className="h-[528px] w-[592px] rounded-[6px] border">
            <div>
              <p className="p-4 text-[14px] font-normal leading-[21px] text-[#262626]">
                First, let&apos;s look at the name of the product. The only version of the product is the main product
                of the small-shaped product. The complete version of the product is signed and the logo is waiting. The
                second version is the main product of the main product of the small-shaped product. The details are
                posted. The right side of the purple is the same as the model lens. The first time is the first time to
                use it, please note whether the model is placed in the correct mode. A very simple simple method, the
                dot-shaped model is the original area. If the line of the line is in the same mode, it means that it is
                in the right mode or it is placed in the correct mode. Once the line of the line is selected, the
                dot-shaped model has a difference. Here is your product image. The image quality of the image quality of
                the image quality is at the top of the tree, such as the height of the tree is 168×1152 or
                960×1536×1152×768 or 960. The image quality of the image quality is at the top of the tree, which is not
                necessary to modify the image quality of the image quality. The image quality of the image quality is at
                the top of the tree, which is not necessary to modify the image quality of the image quality. Please
                adjust the image quality before the image quality is adjusted.\nHere you can find the reference tool you
                like. If the reference tool is not used, it will be cut in the middle. The flow of the flow will
                automatically be transferred to the reference tool, and the reference tool will be transferred to the
                reference tool. The detailed explanation and the control of the flow of the flow will be stored here. If
                you don&apos;t use reference tool, you need to avoid this function, and then the reference tool will be
                transferred to the reference tool. The second option of the keyboard tool is to select the second
                option. The complicated product is not reliable, you need to use the reference tool for the keyboard.
                The complicated product is not reliable, you need to use the reference tool for the keyboard. The
                complicated product is not reliable, you need to use the reference tool for the keyboard.
              </p>
            </div>
          </ScrollArea>
          <MuseBadge className="absolute bottom-[34px] right-[34px]">
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
          <Button className="mt-4 w-full">导出</Button>
        </div>
      </div>
    </div>
  )
}
