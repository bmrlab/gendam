import SpecificContextMenuList from '@/components/FileContent/ContextMenu'
import {
  ContextMenuDelete,
  ContextMenuDetails,
  ContextMenuMove,
  ContextMenuOpen,
  ContextMenuProcessMetadata,
  ContextMenuRename,
  ContextMenuReprocess,
  ContextMenuShare,
  ContextMenuUpload,
} from '@/Explorer/components/ContextMenu'
import { ContextMenu } from '@gendam/ui/v2/context-menu'
import { forwardRef } from 'react'

const ItemContextMenu = forwardRef<HTMLDivElement, {}>(function ItemContextMenuComponent({ ...props }, forwardedRef) {
  return (
    <ContextMenu.Content
      ref={forwardedRef}
      {...props}
      onClick={(e) => e.stopPropagation()}
      className="data-[state=closed]:animate-none data-[state=closed]:duration-0"
    >
      <ContextMenuOpen />
      <ContextMenuDetails />
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenuProcessMetadata />
      <ContextMenuReprocess />

      {/* this introduce different operations for different file types */}
      <SpecificContextMenuList />

      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenuMove />
      <ContextMenuRename />
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenuUpload />
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenuShare />
      <ContextMenu.Separator className="bg-app-line my-1 h-px" />
      <ContextMenuDelete />
    </ContextMenu.Content>
  )
})

export default ItemContextMenu
