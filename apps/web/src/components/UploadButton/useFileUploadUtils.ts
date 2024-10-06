'use client'
import { type UploadButtonResult } from '@/components/UploadButton'
import { useUploadQueueStore } from '@/components/UploadQueue/store'
import { queryClient, rspc } from '@/lib/rspc'
import { useCallback } from 'react'
import { toast } from 'sonner'

type Node = {
  name: string
  fileItems: Extract<UploadButtonResult, { directory: false }>['items']
  childDirs: Node[]
}
const buildDirectoryTree = (items: Extract<UploadButtonResult, { directory: true }>['items']) => {
  const root: Node[] = []
  for (const { relativePath, ...rest } of items) {
    let dirs = root
    let dir: Node | undefined
    // relativePath is like 'a/b/c', without leading '/' and trailing '/'
    for (const name of relativePath.split('/')) {
      dir = dirs.find((n) => n.name === name)
      if (!dir) {
        dir = { name, fileItems: [], childDirs: [] }
        dirs.push(dir)
      }
      dirs = dir.childDirs
    }
    if (dir) {
      // 把文件放到最后一个目录下
      dir.fileItems.push(rest)
    }
  }
  return root
}

export function useFileUploadUtils() {
  const createDirMut = rspc.useMutation(['assets.create_dir'])
  const uploadQueueStore = useUploadQueueStore()
  // 只监听 enqueue，不监听 uploadQueueStore 数据变化，不然这里定义的方法会会一直更新，导致依赖这些方法的 useEffect 会始终被触发
  const enqueue = uploadQueueStore.enqueue

  const enqueueFiles = useCallback(
    (materializedPath: string, fileItems: Extract<UploadButtonResult, { directory: false }>['items']) => {
      for (const item of fileItems) {
        if ('file' in item) {
          enqueue({
            materializedPath,
            name: item.file.name,
            dataType: 'file',
            payload: item.file,
          })
        } else if ('fileSystemPath' in item) {
          const name = item.fileSystemPath.split('/').slice(-1).join('')
          enqueue({
            materializedPath,
            name,
            dataType: 'path',
            payload: item.fileSystemPath,
          })
        }
      }
    },
    [enqueue],
  )

  const createDirectoriesAndEnqueue = useCallback(
    async (materializedPath: string, root: Node[]) => {
      const queue = [{ materializedPath, dirs: root }]
      while (true) {
        const item = queue.shift()
        if (!item) {
          break
        }
        const { materializedPath, dirs } = item
        for (const dir of dirs) {
          console.log('create dir', materializedPath, dir.name)
          // rspc.ts 会处理错误弹出提醒，如果出错，这里直接抛出错误，不继续
          const filePathCreated = await createDirMut.mutateAsync({
            materializedPath,
            name: dir.name,
          })
          // 因为创建的时候会自动重命名，所以这里要用返回的 name
          const newMaterializedPath = `${materializedPath}${filePathCreated.name}/`
          enqueueFiles(newMaterializedPath, dir.fileItems)
          queue.push({ materializedPath: newMaterializedPath, dirs: dir.childDirs })
        }
      }
    },
    [createDirMut, enqueueFiles],
  )

  const handleSelectEventOfUploadButton = useCallback(
    async (materializedPath: string, { items, directory }: UploadButtonResult) => {
      if (directory) {
        const tree = buildDirectoryTree(items)
        try {
          await createDirectoriesAndEnqueue(materializedPath, tree)
        } catch (e) {
          toast.error(`Request Error`, {
            description:
              'Failed to create directories, please delete the new created directories manually and try again',
          })
        }
        queryClient.invalidateQueries({
          queryKey: ['assets.list', { materializedPath }],
        })
      } else {
        enqueueFiles(materializedPath, items)
      }
    },
    [createDirectoriesAndEnqueue, enqueueFiles],
  )

  return { handleSelectEventOfUploadButton }
}
