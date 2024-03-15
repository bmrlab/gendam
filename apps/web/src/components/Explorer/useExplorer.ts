import type { ExplorerItem } from './types'

type UseExplorerSettings = {
  layout: 'grid' | 'list';
}

type UseExplorerProps = {
  items?: ExplorerItem[] | null;
	count?: number;
	parentPath?: string;
  settings: UseExplorerSettings
}

export function useExplorer({
  settings,
	...props
}: UseExplorerProps) {
  return {
    count: props.items?.length ?? 0,
    settings: {
      layout: 'grid',
    },
    ...props,
  }
}

export type UseExplorer = ReturnType<typeof useExplorer>;
