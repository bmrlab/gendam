import { TaskStatus, getTaskStatus } from '@/app/video-tasks/_components/utils'
import { FileHandlerTask } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { useMemo } from 'react'
import { create } from 'zustand'
import useEffectWithPrevDeps from './useEffectWithPrevDeps'

type tasksStoreType = {
  tasks: Map<number, { materializedPath: string; name: string }>
  insert: (assetObjectId: number, task: { materializedPath: string; name: string }) => void
  remove: (assetObjectId: number) => void
}

export const useTasksStore = create<tasksStoreType>((set, get) => ({
  tasks: new Map(),
  insert: (assetObjectId: number, payload: { materializedPath: string; name: string }) => {
    set((state) => {
      const tasks = new Map(state.tasks)
      tasks.set(assetObjectId, payload)
      return { tasks }
    })
  },
  remove: (assetObjectId) => {
    set((state) => {
      const tasks = new Map(state.tasks)
      tasks.delete(assetObjectId)
      return { tasks }
    })
  },
}))

// 任务完成触发的hooks
export const useTasks = () => {
  const taskStore = useTasksStore()
  const taskList: number[] = useMemo(() => {
    return Array.from(taskStore.tasks.keys())
  }, [taskStore.tasks])

  const { data } = rspc.useQuery(['video.tasks.get_tasks', { assetObjectIdList: taskList }], {
    refetchInterval: 5000,
    refetchOnMount: true,
  })

  const { mutateAsync: updateDocNewFile } = rspc.useMutation(['assets.update_doc_new_file'])

  const assetObjectIdWithTasks = useMemo(() => {
    return data?.map((i) => {
      return {
        assetObjectId: i.id,
        // @ts-ignore
        isProcessing: i.tasks?.some((task: FileHandlerTask) => getTaskStatus(task) === TaskStatus.Processing),
      }
    })
  }, [data])

  useEffectWithPrevDeps(
    ([oldAssetObjectIdWithTasks]) => {
      assetObjectIdWithTasks?.forEach((i) => {
        let old = oldAssetObjectIdWithTasks?.find((o) => o.assetObjectId === i.assetObjectId)!
        if (!!old?.isProcessing && !i?.isProcessing) {
          // 发送新增文件crdt
          let task = taskStore.tasks.get(i.assetObjectId)!
          updateDocNewFile({
            assetObjectId: i.assetObjectId,
          })
          // 移除监听任务
          taskStore.remove(i.assetObjectId)
        }
      })
    },
    [assetObjectIdWithTasks, taskStore],
  )
}
