'use client'
import { ExtractExplorerItem } from '@/Explorer/types'
import { Dialog } from '@gendam/ui/v2/dialog'
import { useCallback } from 'react'
import BatchExport from './AudioBatchExport'
import AudioExport from './AudioExport'
import { useBoundStore as useAudioBoundStore } from './store'
import { AudioDialogEnum } from './store/audio-dialog'

export default function AudioDialog() {
  const isOpenAudioDialog = useAudioBoundStore.use.isOpenAudioDialog()
  const audioDialogProps = useAudioBoundStore.use.audioDialogProps()
  const setIsOpenAudioDialog = useAudioBoundStore.use.setIsOpenAudioDialog()

  return (
    <Dialog.Root open={isOpenAudioDialog} onOpenChange={setIsOpenAudioDialog}>
      <Dialog.Portal>
        <Dialog.Overlay onClick={(e) => e.stopPropagation()} />
        <Dialog.Content className="flex w-[70rem] flex-col overflow-hidden">
          <div className="border-app-line border-b py-4 pl-6 leading-5">{audioDialogProps.title}</div>
          {audioDialogProps.type === AudioDialogEnum.single ? <AudioExport /> : <BatchExport />}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}

export const useAudioDialog = () => {
  const setIsOpenAudioDialog = useAudioBoundStore.use.setIsOpenAudioDialog()
  const setAudioDialogProps = useAudioBoundStore.use.setAudioDialogProps()
  const setAudioDialogOpen = useAudioBoundStore.use.setIsOpenAudioDialog()

  const singleExport = useCallback(
    (item: ExtractExplorerItem<'FilePath'>) => {
      setAudioDialogProps({
        type: AudioDialogEnum.single,
        title: 'Export Transcript',
        params: {
          fileHash: item.assetObject.hash,
        },
      })
      setIsOpenAudioDialog(true)
    },
    [setAudioDialogProps, setIsOpenAudioDialog],
  )

  const batchExport = useCallback(
    (items: ExtractExplorerItem<'FilePath'>[]) => {
      items = items.filter(
        (item) => item.assetObject.mediaData?.contentType === 'video' && !!item.assetObject.mediaData.audio,
      )
      // items.sort((a, b) => a.assetObject.id - b.assetObject.id)
      setAudioDialogProps({
        type: AudioDialogEnum.batch,
        title: 'Bulk Transcript Export',
        params: items.map((item) => ({
          id: item.assetObject.hash, // TODO: 这里回头要改成 assetObjectId, 但是对 audio export 功能改动较大
          label: item.filePath.name,
          assetObjectId: item.assetObject.id,
          assetObjectHash: item.assetObject.hash,
        })),
      })
      setAudioDialogOpen(true)
    },
    [setAudioDialogOpen, setAudioDialogProps],
  )

  return { singleExport, batchExport }
}
