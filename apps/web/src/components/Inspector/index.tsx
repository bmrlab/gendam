'use client'

import { ExplorerItem } from '@/Explorer/types'
import InspectorItem from '../FileContent/Inspector'

export default function Inspector({ data }: { data: ExplorerItem | null }) {
  return (
    <div className="border-app-line h-full w-full overflow-auto border-l">
      <InspectorItem data={data} />
    </div>
  )
}
