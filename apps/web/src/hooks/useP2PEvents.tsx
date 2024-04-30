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
      shareInfo: {
        fileCount: number
        docIdHashList: {
          hash: string
          name: string
          docId: string
        }[]
        folderDocIdList: FolderDocIdList
      }
      fileList: {
        size: number
        path: string
      }[]
    }
  | { type: 'ShareProgress'; id: string; percent: number; shareInfo: { fileCount: number } }
  | { type: 'ShareTimedOut'; id: string }
  | { type: 'ShareRejected'; id: string }

type FolderDocIdList = {
  docId: string
  folder: {
    name: string
    children: {
      isDir: boolean
      path: string
    }[]
  }
}[]

export const useP2PEvents = () => {
  const { mutateAsync: acceptFileShare } = rspc.useMutation(['p2p.accept_file_share'])
  const { mutateAsync: rejectFileShare } = rspc.useMutation(['p2p.reject_file_share'])
  const { mutateAsync: finishFileShare } = rspc.useMutation(['p2p.finish_file_share'])
  const { mutateAsync: receiveAsset } = rspc.useMutation(['assets.receive_asset'])
  const { mutateAsync: updateFileAndDoc } = rspc.useMutation(['assets.update_file_and_doc'])
  const { mutateAsync: updateFolderDoc } = rspc.useMutation(['assets.update_folder_doc'])
  const { mutateAsync: createDir } = rspc.useMutation(['assets.create_dir'])

  const progressRef = useRef(new WeakMap())
  const receivePathMap = useRef(new Map<string, string[]>())
  const receiveDirMap = useRef(new Map<string, FilePath | null>())
  const receiveDocMap = useRef(new Map<string, { name: string; docId: string }>())
  const receiveFolderMap = useRef(new Map<string, FolderDocIdList>())

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
                      data.shareInfo.docIdHashList.forEach((v) => {
                        receiveDocMap.current.set(v.hash, {
                          name: v.name,
                          docId: v.docId,
                        })
                      })
                      receiveFolderMap.current.set(data.id, data.shareInfo.folderDocIdList)
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
          const folderDocIdList = receiveFolderMap.current.get(data.id) || []

          if (filePathList) {
            filePathList.forEach((filePath, index) => {
              finishFileShare(filePath).then((res) => {
                // 触发文件处理任务
                const targetDir = receiveDirMap.current.get(data.id)
                // 创建对应的文件夹
                folderDocIdList.forEach((i) => {
                  // 先创建folder文件夹
                  if (!i.folder.name) return
                  let root = targetDir ? `${targetDir.materializedPath}` : '/'
                  let folderList = []
                  folderList.push({
                    name: i.folder.name,
                    materializedPath: root,
                  })

                  if (i.folder.children) {
                    i.folder.children.forEach(async (child) => {
                      if (child.isDir) {
                        // 分割path
                        const path = child.path.split('/')
                        const name = path.pop()
                        let materializedPath = root + i.folder.name + '/'
                        path.forEach((i) => {
                          materializedPath = materializedPath + i + '/'
                        })
                        folderList.push({
                          name,
                          materializedPath,
                        })
                      }
                    })
                  }

                  folderList.forEach(async (v) => {
                    await createDir({
                      name: v.name,
                      materializedPath: v.materializedPath,
                    })
                  })
                })

                res.forEach((v) => {
                  const root = targetDir ? `${targetDir.materializedPath}${targetDir.name}/` : '/'
                  const { name, docId } = receiveDocMap.current.get(v)!

                  let path = root
                  if (!!folderDocIdList?.[index]) {
                    let i = folderDocIdList?.[index]
                    const folderName = i.folder.name
                    const children = i.folder.children
                    children.forEach((v) => {
                      if (!v.isDir) {
                        let childrenPath = v.path
                        let childrenPathList = childrenPath.split('/')
                        if (childrenPathList[childrenPathList.length - 1] === name) {
                          let last = childrenPathList.slice(0, childrenPathList.length - 1).join('/')
                          path = root + folderName + '/' + (!!last ? last + '/' : '')
                        }
                      }
                    })
                  }
                  receiveAsset({
                    hash: v,
                    materializedPath: path,
                    name,
                  }).then(() => {
                    // 创建完成 更新 文档和文件的关联关系
                    if (docId) {
                      updateFileAndDoc({ docId, name, path })
                    }
                  })
                })
                // 更新文件夹的文档
                folderDocIdList.forEach((i) => {
                  let docId = i.docId
                  let name = i.folder.name
                  let materializedPath = targetDir ? `${targetDir.materializedPath}${targetDir.name}/` : '/'
                  setTimeout(() => {
                    // 为什么加setTimeout， 如果只传一个空文件夹过去，就会太快触发这个函数
                    updateFolderDoc({ docId, name, path: materializedPath })
                  }, 100)
                })
                receivePathMap.current.delete(data.id)
                receiveDirMap.current.delete(data.id)
                receiveFolderMap.current.delete(data.id)
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
