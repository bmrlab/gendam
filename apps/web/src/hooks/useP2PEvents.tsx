import { rspc } from '@/lib/rspc'
import { Button } from '@gendam/ui/v2/button'
import { useCallback, useRef } from 'react'
import { toast } from 'sonner'
import { useOpenFileSelection } from './useOpenFileSelection'

type Event =
  | { type: 'ShareRequest'; id: string; peerId: string; peerName: string; files: { name: string; hash: string }[] }
  | { type: 'ShareProgress'; id: string; percent: number }
  | { type: 'ShareTimedOut'; id: string }
  | { type: 'ShareRejected'; id: string }

export const useP2PEvents = () => {
  const { mutateAsync: acceptFileShare } = rspc.useMutation(['p2p.acceptFileShare'])
  const progressRef = useRef(new WeakMap())

  const { openFileSelection } = useOpenFileSelection()

  // todo 取消文件分享
  // const { mutateAsync: cancelFileShare } = rspc.useMutation(['p2p.cancelFileShare'])

  const handleShareRequest = useCallback(
    (data: Event) => {
      if (data.type === 'ShareRequest') {
        // 弹提示窗口
        const toastId = toast.info(`文件分享`, {
          description: `来自${data.peerId.slice(0, 5)}的${data.files.length}个文件分享请求`,
          duration: 60000,
          action: (
            <div className="flex items-center space-x-2">
              <Button
                size="sm"
                onClick={() => {
                  openFileSelection().then((path) => {
                    console.log('path', path)
                    acceptFileShare([data.id, [...data.files.map((i) => i.hash)]])
                    toast.dismiss(toastId)
                  })
                }}
              >
                Accept
              </Button>
              <Button
                size="sm"
                variant="destructive"
                onClick={() => {
                  acceptFileShare([data.id, null])
                  toast.dismiss(toastId)
                }}
              >
                Reject
              </Button>
            </div>
          ),
        })
      }

      if (data.type === 'ShareProgress') {
        progressRef.current.set({ id: data.id }, { percent: data.percent, id: data.id, show: true })
        if (data.percent === 100) {
          progressRef.current.set({ id: data.id }, { percent: data.percent, id: data.id, show: false })
          toast.success('File share completed')
        }
      }

      if (data.type === 'ShareRejected') {
        // 弹提示窗口 被拒绝
        toast.error('Share rejected')
      }
    },
    [acceptFileShare],
  )

  rspc.useSubscription(['p2p.events'], {
    onData(data: Event) {
      console.log('useP2PEvents', data)
      handleShareRequest(data)
    },
  })
}
