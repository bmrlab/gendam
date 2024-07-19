'use client'

import { Button } from '@gendam/ui/v2/button'
import { useState } from 'react'
import { SearchResultPayload } from '../context'
import { RAGReferenceContent, RAGReferencePreview } from './ReferenceResult'

interface RAGReferenceListProps {
  items: SearchResultPayload[]
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
          const key = `${item.filePath.id}-${item.metadata.score}`
          return expand ? <RAGReferenceContent key={key} item={item} /> : <RAGReferencePreview key={key} item={item} />
        })}
      </div>
    </div>
  )
}
