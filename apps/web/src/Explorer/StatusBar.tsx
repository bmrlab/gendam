'use client'
import { useExplorerContext } from './Context'

export default function Explorer() {
  const explorer = useExplorerContext()

  return <div>{explorer.count}</div>
}
