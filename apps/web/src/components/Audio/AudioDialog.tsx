'use client'
import BatchExport from './AudioBatchExport'
import AudioExport from './AudioExport'
import { AudioDialogEnum } from './store/audio-dialog'
import { Dialog } from '@gendam/ui/v2/dialog'
import classNames from 'classnames'
import { useBoundStore } from '../../app/video-tasks/_store'

export default function AudioDialog() {
  const isOpenAudioDialog = useBoundStore.use.isOpenAudioDialog()
  const audioDialogProps = useBoundStore.use.audioDialogProps()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()

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
