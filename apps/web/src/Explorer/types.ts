import type { FilePath, SearchResultMetadata } from '@/lib/bindings'

export type ExplorerItem =
  | {
      type: 'FilePath'
      filePath: FilePath
    }
  | {
      type: 'SearchResult'
      filePath: FilePath
      metadata: SearchResultMetadata
    }
  | {
      // ensure there is no default case
      type: 'Unknown'
    }

export function uniqueId(item: ExplorerItem): string {
  switch (item.type) {
    case 'FilePath':
      return `FilePath:${item.filePath.id}`
    case 'SearchResult':
      return `SearchResult:${item.filePath.id}:${item.metadata.startTime}`
    case 'Unknown':
      return `Unknown:${Math.random()}`
  }
}

// function filtered<K extends ExplorerItem['type'], T extends Extract<ExplorerItem, K>>(
//   items: ExplorerItem[],
//   types: ExplorerItem['type'][],
// ): T[] {
//   return items.filter((item) => types.includes(item.type)) as T[]
// }
