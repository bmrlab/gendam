'use client'
import Icon from '@/components/Icon'
import MuseInput from '@/components/Input'
import { cn, twx } from '@/lib/utils'
import { useBoundStore } from '../_store'
import { HTMLAttributes } from 'react'

export default function VideoTaskHeader({ className }: HTMLAttributes<HTMLDivElement>) {
  const searchKey = useBoundStore.use.searchKey()
  const setSearchKey = useBoundStore.use.setSearchKey()

  return (
    <div className={cn('flex items-center justify-between border-b border-neutral-100 px-4', className)}>
      <div className="flex select-none items-center gap-2">
        <Icon.arrowLeft className="size-6 cursor-pointer text-[#797979]" />
        <Icon.arrowRight className="size-6 cursor-pointer text-[#797979]" />
        <div className="text-[14px] font-medium leading-[18px] text-[#232526]">任务列表</div>
      </div>
      <MuseInput
        value={searchKey}
        onChange={(e) => setSearchKey(e.target.value ?? '')}
        icon={Icon.search}
        placeholder="搜索"
      />
      <div className="flex items-center gap-0.5 justify-self-end text-[#676C77]">
        <IconButton>
          <Icon.grid className="size-4 text-[#797979]" />
        </IconButton>
        <IconButton>
          <Icon.list className="size-4 text-[#797979]" />
        </IconButton>
        <IconButton>
          <Icon.column className="size-4 text-[#797979]" />
        </IconButton>
      </div>
    </div>
  )
}

const IconButton = twx.div`h-6 w-[28px] cursor-pointer rounded px-1.5 py-1 hover:bg-[#EBECEE]`
