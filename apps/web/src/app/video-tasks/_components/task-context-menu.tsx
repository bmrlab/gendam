import { AudioDialogEnum } from '@/app/video-tasks/store/audio-dialog'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { useBoundStore } from '@/store'
import { PropsWithChildren, useCallback, useMemo } from 'react'

export type TaskContextMenuProps = PropsWithChildren<{
  fileHash: string
}>

export default function TaskContextMenu({ fileHash, children }: TaskContextMenuProps) {
  const taskSelected = useBoundStore.use.taskSelected()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()
  const setAudioDialogProps = useBoundStore.use.setAudioDialogProps()
  const setAudioDialogOpen = useBoundStore.use.setIsOpenAudioDialog()

  const isBatchSelected = useMemo(() => taskSelected.length > 1, [taskSelected])

  const handleSingleExport = useCallback(() => {
    setAudioDialogProps({
      type: AudioDialogEnum.single,
      title: '导出语音转译',
      params: {
        fileHash,
      },
    })
    setIsOpenAudioDialog(true)
  }, [fileHash, setAudioDialogProps, setIsOpenAudioDialog])

  const handleBatchExport = () => {
    let orderTaskSelected = [...taskSelected]
    orderTaskSelected.sort((a, b) => a.index - b.index)
    setAudioDialogProps({
      type: AudioDialogEnum.batch,
      title: '批量导出语音转译',
      params: orderTaskSelected.map((item) => ({
        id: item.videoFileHash,
        label: item.videoFileHash,
        video: item.videoPath,
      })),
    })
    setAudioDialogOpen(true)
  }

  const handleExport = isBatchSelected ? handleBatchExport : handleSingleExport

  return (
    <ContextMenu>
      <ContextMenuTrigger className="flex cursor-default items-center justify-center rounded-md text-sm">
        {children}
      </ContextMenuTrigger>
      <ContextMenuContent className="w-[215px]">
        <ContextMenuItem inset>重新触发任务</ContextMenuItem>
        <ContextMenuItem inset onClick={handleExport}>
          导出语音转译
        </ContextMenuItem>
        <ContextMenuSeparator />
        <ContextMenuItem inset>删除任务</ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  )
}
