'use client'
import BatchExport from './batch-export'
import AudioExport from './export'
import { AudioDialogEnum } from '@/app/video-tasks/store/audio-dialog'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { useBoundStore } from '@/store'

export default function AudioDialog() {
  const isOpenAudioDialog = useBoundStore.use.isOpenAudioDialog()
  const audioDialogProps = useBoundStore.use.audioDialogProps()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()

  return (
    <Dialog open={isOpenAudioDialog} onOpenChange={setIsOpenAudioDialog}>
      <DialogContent closeHidden overlayClassName="bg-white/90" className="w-[920px] max-w-full gap-0 p-0 shadow-lg">
        <DialogHeader className="border-b py-4">
          <DialogTitle className="pl-6 text-[16px] font-medium leading-[22.4px] text-[#262626]">
            {audioDialogProps.title}
          </DialogTitle>
        </DialogHeader>
        {audioDialogProps.type === AudioDialogEnum.single ? <AudioExport /> : <BatchExport />}
      </DialogContent>
    </Dialog>
  )
}
