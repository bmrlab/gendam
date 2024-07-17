'use client'
import { type ExplorerItem } from '@/Explorer/types'
import type { FilePath, Trash } from '@/lib/bindings'
import { FC, memo, useMemo } from 'react'

type T = Extract<ExplorerItem, { filePath: FilePath | Trash }>

export const ExpiredTag: FC<{ data: T }> = memo(({ data }) => {
  const timeRemaining = useMemo(() => {
    const createdAt = new Date(data.filePath.createdAt)
    const expiryDate = new Date(createdAt.getTime() + 60 * 24 * 60 * 60 * 1000)
    const now = new Date()
    const timeDiff = expiryDate.getTime() - now.getTime()

    if (timeDiff <= 0) {
      return 'Expired'
    }
    const daysRemaining = Math.floor(timeDiff / (1000 * 60 * 60 * 24))
    const hoursRemaining = Math.floor((timeDiff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60))
    const minutesRemaining = Math.floor((timeDiff % (1000 * 60 * 60)) / (1000 * 60))
    const secondsRemaining = Math.floor((timeDiff % (1000 * 60)) / 1000)

    if (daysRemaining > 0) {
      return `${daysRemaining + 1} days`
    } else if (hoursRemaining > 0) {
      return `${hoursRemaining  + 1} hours`
    } else if (minutesRemaining > 0) {
      return `${minutesRemaining  + 1} minutes`
    } else {
      return `${secondsRemaining} seconds`
    }
  }, [data.filePath.createdAt])

  if (!data.filePath.hasOwnProperty('originParentId')) {
    return null
  }

  return (
    <div className="pointer-events-none absolute right-0 top-0 z-10">
      <div className="rounded bg-orange-500 px-1 py-0.5 text-xs text-white">{timeRemaining}</div>
    </div>
  )
})

ExpiredTag.displayName = 'ExpiredTag'
