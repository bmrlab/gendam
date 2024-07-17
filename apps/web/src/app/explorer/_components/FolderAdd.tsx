import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import { useTitleDialog } from './TitleDialog'

export const FolderAdd = () => {
  const titleDialog = useTitleDialog()
  return (
    <Button variant="ghost" size="sm" className="h-7 w-7 p-1 transition-none" onClick={() => titleDialog.setOpen(true)}>
      <Icon.FolderAdd className="size-4" />
    </Button>
  )
}
