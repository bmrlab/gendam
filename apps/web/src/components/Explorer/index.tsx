'use client'
import { useExplorerContext } from './Context'
import GridView from './Layout/GridView'

export default function Explorer() {
  const explorer = useExplorerContext()

  if (!explorer.items || explorer.items.length === 0) {
    return (
      <div>
        <h1>No items</h1>
      </div>
    )
  }

  return (
    <>
      <GridView items={explorer.items}></GridView>
    </>
  )
}
