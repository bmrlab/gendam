import { rspc } from '@/lib/rspc'
import { Button } from '@gendam/ui/v2/button'
import { useCallback, useRef } from 'react'
import { toast } from 'sonner'
import { useOpenFileSelection } from './useOpenFileSelection'

type Event =
  | { type: 'ShareRequest'; id: string; peerId: string; peerName: string; files: { name: string; hash: string }[] }
  | { type: 'ShareProgress'; id: string; percent: number; files: string[] }
  | { type: 'ShareTimedOut'; id: string }
  | { type: 'ShareRejected'; id: string }

export const useP2PEvents = () => {
  const { mutateAsync: acceptFileShare } = rspc.useMutation(['p2p.acceptFileShare'])
  const { mutateAsync: receiveAsset } = rspc.useMutation(['assets.receive_asset'])
  const progressRef = useRef(new WeakMap())

  // FIXME temporary test code
  const receiveHashes = useRef(new Set())

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
                    // FIXME temporary test code
                    data.files.forEach((v) => {
                      receiveHashes.current.add(v.hash)
                    })
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
        console.log('receiveHashes', receiveHashes.current)
        progressRef.current.set({ id: data.id }, { percent: data.percent, id: data.id, show: true })
        if (data.percent === 100) {
          progressRef.current.set({ id: data.id }, { percent: data.percent, id: data.id, show: false })
          toast.success('File share completed')
          console.log('receiveHashes', receiveHashes.current)
          // 触发文件处理任务
          data.files.forEach((hash) => {
            console.log('hash', hash)
            if (receiveHashes.current.has(hash)) {
              receiveHashes.current.delete(hash)
              console.log("trigger receive asset")
              receiveAsset({ hash })
            }
          })
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
