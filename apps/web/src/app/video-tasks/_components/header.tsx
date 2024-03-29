'use client'
import Icon from '@/components/Icon'
import MuseInput from '@/components/Input'
import { cn, twx } from '@/lib/utils'
import { useBoundStore } from '../_store'
import { HTMLAttributes } from 'react'
import Viewport from '@/components/Viewport'
import PageNav from '@/components/PageNav'

export default function VideoTaskHeader({ className }: HTMLAttributes<HTMLDivElement>) {
  const searchKey = useBoundStore.use.searchKey()
  const setSearchKey = useBoundStore.use.setSearchKey()

  return (
    <Viewport.Toolbar className="items-center justify-between">
      <PageNav title="任务列表" />
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
    </Viewport.Toolbar>
  )
}

const IconButton = twx.div`h-6 w-[28px] cursor-pointer rounded px-1.5 py-1 hover:bg-[#EBECEE]`
