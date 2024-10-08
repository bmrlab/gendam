'use client'

import { ExtractExplorerItem, uniqueId } from '@/Explorer/types'
import { Button } from '@gendam/ui/v2/button'
import { useState } from 'react'
import RetrievalResultItem, { RetrievalResultItemPreview } from './RAGRetrievalResultItem'

interface RAGReferenceListProps {
  items: ExtractExplorerItem<'RetrievalResult'>[]
  isLoading: boolean
}

export function RAGReferenceList({ items, isLoading }: RAGReferenceListProps) {
  const [expand, setExpand] = useState(false)

  return (
    <div className="w-full overflow-hidden">
      {(isLoading || items.length > 0) && (
        <div className="flex w-full justify-between">
          <h2 className="text-lg font-bold">Related Assets</h2>
          <Button onClick={() => setExpand((v) => !v)}>{expand ? 'Collapse' : 'Expand'}</Button>
        </div>
      )}

      <div
        className="mt-4 flex w-full flex-row space-x-4 overflow-x-scroll aria-expanded:flex-col aria-expanded:space-x-0 aria-expanded:space-y-8"
        aria-expanded={expand}
      >
        {items.map((item) => {
          return expand ? (
            <RetrievalResultItem key={uniqueId(item)} {...item} />
          ) : (
            <div
              className="border-app-line group h-[160px] w-[240px] shrink-0 overflow-hidden rounded-lg border-2"
              key={uniqueId(item)}
            >
              <RetrievalResultItemPreview {...item} />
            </div>
          )
        })}
      </div>
    </div>
  )
}
