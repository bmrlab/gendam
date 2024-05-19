import { type FilePath } from '@/lib/bindings'
import { websocketClient, rspc } from '@/lib/rspc'
import { Button } from '@gendam/ui/v2/button'
import { useCallback, useRef } from 'react'
import { toast } from 'sonner'
import { useOpenFileSelection } from './useOpenFileSelection'

type Event =
  | {
      type: 'ShareRequest'
      id: string
      peerId: string
      peerName: string
      shareInfo: { fileCount: number }
      fileList: {
        size: number
        path: string
      }[]
    }
  | { type: 'ShareProgress'; id: string; percent: number; shareInfo: { fileCount: number } }
  | { type: 'ShareTimedOut'; id: string }
  | { type: 'ShareRejected'; id: string }

export const useP2PEvents = () => {
  const { mutateAsync: acceptFileShare } = rspc.useMutation(['p2p.accept_file_share'])
  const { mutateAsync: rejectFileShare } = rspc.useMutation(['p2p.reject_file_share'])
  const { mutateAsync: finishFileShare } = rspc.useMutation(['p2p.finish_file_share'])
  const { mutateAsync: receiveAsset } = rspc.useMutation(['assets.receive_asset'])

  const progressRef = useRef(new WeakMap())
  const receivePathMap = useRef(new Map<string, string[]>())
  const receiveDirMap = useRef(new Map<string, FilePath | null>())

  const { openFileSelection } = useOpenFileSelection()

  // todo 取消文件分享
  // const { mutateAsync: cancelFileShare } = rspc.useMutation(['p2p.cancelFileShare'])

  const handleShareRequest = useCallback(
    (data: Event) => {
      if (data.type === 'ShareRequest') {
        // 弹提示窗口
        const toastId = toast.info(`Asset Share`, {
          description: `${data.peerId.slice(0, 5)} wants to share ${data.shareInfo.fileCount} assets to you`,
          duration: 60000,
          action: (
            <div className="flex items-center space-x-2">
              <Button
                size="sm"
                onClick={() => {
                  openFileSelection().then((path) => {
                    acceptFileShare(data.id).then((res) => {
                      receivePathMap.current.set(data.id, res.fileList)
                      receiveDirMap.current.set(data.id, path)
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
                  rejectFileShare(data.id)
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

          const filePathList = receivePathMap.current.get(data.id)
          if (filePathList) {
            filePathList.forEach((filePath) => {
              finishFileShare(filePath).then((res) => {
                // 触发文件处理任务
                const targetDir = receiveDirMap.current.get(data.id)
                res.forEach((v) => {
                  receiveAsset({
                    hash: v,
                    materializedPath: targetDir ? `${targetDir.materializedPath}${targetDir.name}/` : '/',
                  })
                })
                receivePathMap.current.delete(data.id)
                receiveDirMap.current.delete(data.id)
              })
            })
          }
        }
      }

      if (data.type === 'ShareRejected') {
        // 弹提示窗口 被拒绝
        toast.error('Share rejected')
      }
    },
    [acceptFileShare, finishFileShare, openFileSelection, receiveAsset, rejectFileShare],
  )

  // rspc.useSubscription(['p2p.events'], {
  //   onData(data: Event) {
  //     handleShareRequest(data)
  //   },
  // })
  websocketClient.addSubscription(['p2p.events', undefined as any], {
    onData(data: Event) {
      handleShareRequest(data)
    },
  })
}
