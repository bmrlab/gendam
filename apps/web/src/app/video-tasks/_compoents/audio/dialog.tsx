import AudioExport from '@/app/video-tasks/_compoents/audio/export'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { useBoundStore } from '@/store'

export default function AudioDialog() {
  const isOpenAudioDialog = useBoundStore.use.isOpenAudioDialog()
  const setIsOpenAudioDialog = useBoundStore.use.setIsOpenAudioDialog()

  return (
    <Dialog open={isOpenAudioDialog} onOpenChange={setIsOpenAudioDialog}>
      <DialogContent overlayClassName="bg-white/90" className="w-[920px] max-w-full gap-0 p-0 shadow-lg">
        <DialogHeader className="border-b py-4">
          <DialogTitle className="pl-6 text-[16px] font-medium leading-[22.4px] text-[#262626]">
            导出语音转译
          </DialogTitle>
        </DialogHeader>
        <AudioExport />
      </DialogContent>
    </Dialog>
  )
}
