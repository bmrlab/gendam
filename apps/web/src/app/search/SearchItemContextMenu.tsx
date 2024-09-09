import VideoExportContextMenu from '@/components/FileContent/ContextMenu/ExportVideoSegment'
import { ContextMenuFindSimilar, ContextMenuOpen, ContextMenuReveal } from '@/Explorer/components/ContextMenu'
import { ExtractExplorerItem } from '@/Explorer/types'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { forwardRef } from 'react'

type SearchItemContextMenuProps = {
  data: ExtractExplorerItem<'SearchResult'>
}

const SearchItemContextMenu = forwardRef<typeof ContextMenu.Content, SearchItemContextMenuProps>(
  function SearchItemContextMenuComponent({ data, ...props }, forwardedRef) {
    return (
      <ContextMenu.Content ref={forwardedRef as any} {...props} onClick={(e) => e.stopPropagation()}>
        <ContextMenuOpen />
        <ContextMenuReveal />
        <ContextMenu.Separator />
        <ContextMenuFindSimilar />
        <ContextMenu.Separator />
        <VideoExportContextMenu />
      </ContextMenu.Content>
    )
  },
)

export default SearchItemContextMenu
