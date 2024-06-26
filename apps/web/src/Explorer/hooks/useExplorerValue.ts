import { useCallback, useMemo, useRef, useState } from 'react'
import { uniqueId, type ExplorerItem } from '../types'

type UseExplorerSettings = {
  layout: 'grid' | 'list' | 'media'
}

function useSettings(defaultSettings: UseExplorerSettings) {
  const [settings, setSettings] = useState<UseExplorerSettings>(defaultSettings)

  const update = useCallback(
    (partialSettings: Partial<UseExplorerSettings>) => {
      setSettings({ ...settings, ...partialSettings })
    },
    [settings, setSettings],
  )

  return {
    ...settings,
    update: update,
  }
}

function useSelectedItems(items: ExplorerItem[] | null) {
  const itemIdsWeakMap = useRef(new WeakMap<ExplorerItem, string>())
  const [selectedItemIds, setSelectedItemIds] = useState(() => ({
    value: new Set<string>(),
  }))

  const updateIds = useCallback(() => setSelectedItemIds((h) => ({ ...h })), [setSelectedItemIds])

  const itemsMap = useMemo(
    () =>
      (items ?? []).reduce((items, item) => {
        const id = itemIdsWeakMap.current.get(item) ?? uniqueId(item)
        itemIdsWeakMap.current.set(item, id)
        items.set(id, item)
        return items
      }, new Map<string, ExplorerItem>()),
    [items],
  )

  const selectedItems = useMemo(
    () =>
      Array.from(selectedItemIds.value).reduce((items, id) => {
        const item = itemsMap.get(id)
        if (item) items.add(item)
        return items
      }, new Set<ExplorerItem>()),
    [itemsMap, selectedItemIds],
  )

  return {
    selectedItems,
    selectedItemIds,
    addSelectedItemById: useCallback(
      (newId: string) => {
        selectedItemIds.value.add(newId)
        updateIds()
      },
      [selectedItemIds.value, updateIds],
    ),
    addSelectedItem: useCallback(
      (item: ExplorerItem) => {
        selectedItemIds.value.add(uniqueId(item))
        updateIds()
      },
      [selectedItemIds.value, updateIds],
    ),
    removeSelectedItemById: useCallback(
      (removeId: string) => {
        selectedItemIds.value.delete(removeId)
        updateIds()
      },
      [selectedItemIds.value, updateIds],
    ),
    removeSelectedItem: useCallback(
      (item: ExplorerItem) => {
        selectedItemIds.value.delete(uniqueId(item))
        updateIds()
      },
      [selectedItemIds.value, updateIds],
    ),
    resetSelectedItems: useCallback(
      (items?: ExplorerItem[]) => {
        selectedItemIds.value.clear()
        items?.forEach((item) => selectedItemIds.value.add(uniqueId(item)))
        updateIds()
      },
      [selectedItemIds.value, updateIds],
    ),
    isItemSelected: useCallback((item: ExplorerItem) => selectedItems.has(item), [selectedItems]),
  }
}

type UseExplorerProps = {
  count?: number
  items: ExplorerItem[] | null
  materializedPath?: string
  settings: UseExplorerSettings
}

export function useExplorerValue({ count, items, materializedPath, settings }: UseExplorerProps) {
  return {
    count: count ? count : items?.length ?? 0,
    items,
    materializedPath,
    ...useSelectedItems(items),
    settings: useSettings(settings),
  }
}

export type ExplorerValue = ReturnType<typeof useExplorerValue>
