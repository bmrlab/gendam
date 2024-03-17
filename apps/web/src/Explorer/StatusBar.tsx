'use client'
import { useExplorerContext } from './hooks/useExplorerContext'

export default function Explorer() {
  const explorer = useExplorerContext()

  return <div>{explorer.count}</div>
}
