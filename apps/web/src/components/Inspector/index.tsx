'use client'

import { ExplorerItem } from '@/Explorer/types'
import { ForwardedRef, forwardRef } from 'react'
import InspectorItem from '../FileContent/Inspector'

function Inspector({ data }: { data: ExplorerItem | null }, ref: ForwardedRef<HTMLDivElement>) {
  return (
    <div className="flex h-full w-full overflow-auto">
      <div ref={ref} className="group flex h-full w-2 flex-none justify-center hover:cursor-col-resize">
        <div className="bg-app-line group-hover:bg-app-hover h-full w-[1px] group-hover:w-[4px]" />
      </div>
      <InspectorItem data={data} />
    </div>
  )
}

export default forwardRef(Inspector)
