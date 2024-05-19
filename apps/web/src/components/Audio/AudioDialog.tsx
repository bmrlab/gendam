'use client'
import BatchExport from './AudioBatchExport'
import AudioExport from './AudioExport'
import { AudioDialogEnum } from './store/audio-dialog'
import { Dialog } from '@gendam/ui/v2/dialog'
import { type FilePath } from '@/lib/bindings'
import { useBoundStore as useAudioBoundStore } from './store'
import { useCallback } from 'react'

export default function AudioDialog() {
  const isOpenAudioDialog = useAudioBoundStore.use.isOpenAudioDialog()
  const audioDialogProps = useAudioBoundStore.use.audioDialogProps()
  const setIsOpenAudioDialog = useAudioBoundStore.use.setIsOpenAudioDialog()

  return (
    <Dialog.Root open={isOpenAudioDialog} onOpenChange={setIsOpenAudioDialog}>
      <Dialog.Portal>
        <Dialog.Overlay onClick={(e) => e.stopPropagation()} />
        <Dialog.Content className="w-[70rem] flex flex-col overflow-hidden">
          <div className="border-b border-app-line py-4 pl-6 leading-5">
            {audioDialogProps.title}
          </div>
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

  const singleExport = useCallback((item: FilePath) => {
    if (!item.assetObject) {
      return
    }
    setAudioDialogProps({
      type: AudioDialogEnum.single,
      title: 'Export Transcript',
      params: {
        fileHash: item.assetObject.hash,
      },
    })
    setIsOpenAudioDialog(true)
  }, [setAudioDialogProps, setIsOpenAudioDialog])

  const batchExport = useCallback((items: FilePath[]) => {
    items = items.filter(item => item.assetObject?.mediaData?.hasAudio)
    // items.sort((a, b) => a.assetObject.id - b.assetObject.id)
    setAudioDialogProps({
      type: AudioDialogEnum.batch,
      title: 'Bulk Transcript Export',
      params: items.map((item) => ({
        id: item.assetObject!.hash, // TODO: 这里回头要改成 assetObjectId, 但是对 audio export 功能改动较大
        label: item.name,
        assetObjectId: item.assetObject!.id,
        assetObjectHash: item.assetObject!.hash,
      })),
    })
    setAudioDialogOpen(true)
  }, [setAudioDialogOpen, setAudioDialogProps])

  return { singleExport, batchExport }
}
