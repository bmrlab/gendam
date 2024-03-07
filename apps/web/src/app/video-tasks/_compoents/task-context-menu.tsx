import { AudioDialogEnum } from '@/app/video-tasks/store/audio-dialog'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { useBoundStore } from '@/store'
import { PropsWithChildren, useCallback } from 'react'

export type TaskContextMenuProps = PropsWithChildren<{
  fileHash: string
}>

export default function TaskContextMenu({ fileHash, children }: TaskContextMenuProps) {
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()
  const setAudioDialogProps = useBoundStore.use.setAudioDialogProps()

  const handleExportAudio = useCallback(() => {
    setAudioDialogProps({
      type: AudioDialogEnum.single,
      title: '导出语音转译',
      params: {
        fileHash,
      },
    })
    setIsOpenAudioDialog(true)
  }, [fileHash, setAudioDialogProps, setIsOpenAudioDialog])

  return (
    <ContextMenu>
      <ContextMenuTrigger className="flex cursor-default items-center justify-center rounded-md text-sm">
        {children}
      </ContextMenuTrigger>
      <ContextMenuContent className="w-[215px]">
        <ContextMenuItem inset>重新触发任务</ContextMenuItem>
        <ContextMenuItem inset onClick={handleExportAudio}>
          导出语音转译
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem inset>删除任务</ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  )
}
