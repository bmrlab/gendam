import Icon from '@/components/Icon'
import { useToast } from '@/components/Toast/use-toast'
import type { VideoWithTasksResult } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import {
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuRoot,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@muse/ui/v1/context-menu'
import { PropsWithChildren, ReactNode, useCallback, useMemo } from 'react'
import { useBoundStore } from '../_store'
import { AudioDialogEnum } from '../_store/audio-dialog'

export type TaskContextMenuProps = PropsWithChildren<{
  fileHash: string
  isProcessing: boolean
  video: VideoWithTasksResult
}>

export default function TaskContextMenu({ video, fileHash, isProcessing, children }: TaskContextMenuProps) {
  const { toast } = useToast()
  const videoSelected = useBoundStore.use.videoSelected()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()
  const setAudioDialogProps = useBoundStore.use.setAudioDialogProps()
  const setAudioDialogOpen = useBoundStore.use.setIsOpenAudioDialog()

  const { mutateAsync: regenerateTask } = rspc.useMutation(['video.tasks.regenerate'])

  const isBatchSelected = useMemo(() => videoSelected.length > 1, [videoSelected])

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
    let orderVideoSelected = [...videoSelected]
    orderVideoSelected.sort((a, b) => a.assetObjectId - b.assetObjectId)
    setAudioDialogProps({
      type: AudioDialogEnum.batch,
      title: '批量导出语音转译',
      params: orderVideoSelected.map((item) => ({
        id: item.assetObjectHash, // TODO: 这里回头要改成 assetObjectId, 但是对 audio export 功能改动较大
        label: item.name,
        assetObjectId: item.assetObjectId,
        assetObjectHash: item.assetObjectHash,
      })),
    })
    setAudioDialogOpen(true)
  }

  const handleExport = isBatchSelected ? handleBatchExport : handleSingleExport

  const options = useMemo<Array<'Separator' | { label: string; icon: ReactNode; handleClick: () => void }>>(() => {
    const processingItem = isProcessing
      ? [
          {
            label: '取消任务',
            icon: <Icon.cancel />,
            handleClick: () => {
              console.log('取消任务')
            },
          },
        ]
      : []

    return [
      {
        label: '重新触发任务',
        icon: <Icon.regenerate />,
        handleClick: async () => {
          const res = await regenerateTask({
            materializedPath: video.materializedPath,
            assetObjectId: video.assetObjectId,
          })
          toast({
            title: res ? '重新触发任务成功' : '重新触发任务失败',
            variant: res ? 'default' : 'destructive',
          })
        },
      },
      ...processingItem,
      {
        label: '导出语音转译',
        icon: <Icon.download />,
        handleClick: handleExport,
      },
      'Separator',
      {
        label: '删除任务',
        icon: <Icon.trash />,
        handleClick: () => {
          console.log('删除任务')
        },
      },
    ]
  }, [handleExport, isProcessing])

  return (
    <ContextMenuRoot>
      <ContextMenuTrigger className="flex cursor-default items-center justify-center rounded-md text-sm">
        {children}
      </ContextMenuTrigger>
      <ContextMenuContent className="muse-border w-[215px] bg-[#F4F5F5] py-2 shadow-md">
        {options.map((o, index) =>
          o === 'Separator' ? (
            <ContextMenuSeparator key={index} className="mx-2.5 bg-[#DDDDDE]" />
          ) : (
            <ContextMenuItem
              key={index}
              inset
              className="flex gap-1.5 px-2.5 py-[3.5px] text-[13px] leading-[18.2px] transition focus:bg-[#017AFF] focus:text-white data-[disabled]:text-[#AAADB2] data-[disabled]:opacity-100"
              onClick={o.handleClick}
            >
              {o.icon}
              <span>{o.label}</span>
            </ContextMenuItem>
          ),
        )}
      </ContextMenuContent>
    </ContextMenuRoot>
  )
}
