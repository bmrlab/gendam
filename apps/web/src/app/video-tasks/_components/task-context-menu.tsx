import { AudioDialogEnum } from '../_store/audio-dialog'
import Icon from '@/components/Icon'
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from '@/components/ui/context-menu'
import { useBoundStore } from '../_store'
import { PropsWithChildren, ReactNode, useCallback, useMemo } from 'react'

export type TaskContextMenuProps = PropsWithChildren<{
  fileHash: string
  isProcessing: boolean
}>

export default function TaskContextMenu({ fileHash, isProcessing, children }: TaskContextMenuProps) {
  const videoSelected = useBoundStore.use.videoSelected()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()
  const setAudioDialogProps = useBoundStore.use.setAudioDialogProps()
  const setAudioDialogOpen = useBoundStore.use.setIsOpenAudioDialog()

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
        id: item.assetObjectHash,  // TODO: 这里回头要改成 assetObjectId, 但是对 audio export 功能改动较大
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
        handleClick: () => {
          console.log('重新触发任务')
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
    <ContextMenu>
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
    </ContextMenu>
  )
}
