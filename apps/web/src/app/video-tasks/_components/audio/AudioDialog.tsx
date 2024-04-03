'use client'
import BatchExport from './batch-export'
import AudioExport from './export'
import { AudioDialogEnum } from '../../_store/audio-dialog'
// import { DialogRoot, DialogBody, DialogHeader, DialogTitle } from '@muse/ui/v1/dialog'
import { DialogPrimitive as Dialog } from '@muse/ui/v1/dialog'
import classNames from 'classnames'
import { useBoundStore } from '../../_store'

export default function AudioDialog() {
  const isOpenAudioDialog = useBoundStore.use.isOpenAudioDialog()
  const audioDialogProps = useBoundStore.use.audioDialogProps()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()

  return (
    <Dialog.Root open={isOpenAudioDialog} onOpenChange={setIsOpenAudioDialog}>
      <Dialog.Portal>
        <Dialog.Overlay
          className="fixed inset-0 z-50 bg-black/30  data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0"
          onClick={(e) => e.stopPropagation()}
        />
        <Dialog.Content className={classNames(
          'fixed z-50 left-[50%] top-[50%] w-[60rem] max-w-full translate-x-[-50%] translate-y-[-50%] overflow-auto',
          'text-ink bg-app-box border border-app-line shadow-lg rounded-lg',
          'duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%]',
        )}>
          <div className="border-b border-app-line py-4 pl-6 leading-5">
            {audioDialogProps.title}
          </div>
          {audioDialogProps.type === AudioDialogEnum.single ? <AudioExport /> : <BatchExport />}
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  )
}
