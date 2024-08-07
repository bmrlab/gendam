'use client'
import { matchExplorerItemWithType } from '@/Explorer/pattern'
import { ExtractExplorerItem, RawFilePath } from '@/Explorer/types'
import type { FileHandlerTask, FilePath } from '@/lib/bindings'
import { formatDateTime } from '@/lib/utils'
import { Folder_Light } from '@gendam/assets/images'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import Image from 'next/image'
import { useEffect } from 'react'
import { match } from 'ts-pattern'
import AudioDetail from '../FileContent/Inspector/Audio'
import VideoDetail from '../FileContent/Inspector/Video'
import { useSortedTasks } from './hooks'
import { useInspector } from './store'

const FolderDetail = ({ data }: { data: RawFilePath }) => {
  return (
    <div className="p-4">
      <div className="flex items-start justify-start">
        <div className="relative h-12 w-12">
          <Image src={Folder_Light} alt="folder" fill={true} className="object-contain"></Image>
        </div>
        <div className="ml-3 flex-1 overflow-hidden">
          <div className="text-ink mt-1 line-clamp-2 text-xs font-medium">{data.name}</div>
          {/* <div className="line-clamp-2 text-ink/50 text-xs mt-1">文件夹 {data.materializedPath}{data.name}</div> */}
        </div>
      </div>
      <div className="bg-app-line mb-3 mt-6 h-px"></div>
      <div className="text-xs">
        <div className="text-md font-medium">Information</div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Created</div>
          <div>{formatDateTime(data.createdAt)}</div>
        </div>
        <div className="mt-2 flex justify-between">
          <div className="text-ink/50">Modified</div>
          <div>{formatDateTime(data.updatedAt)}</div>
        </div>
      </div>
    </div>
  )
}

export function TaskItemStatus({ task }: { task: FileHandlerTask }) {
  if (task.exitCode !== null && task.exitCode > 1) {
    return <Icon.Close className="h-3 w-3 text-red-500" /> // 出错
  } else if (task.exitCode === 1) {
    return <Icon.Close className="h-3 w-3 text-neutral-500" /> // 取消
  } else if (task.exitCode === 0) {
    return <Icon.Check className="h-3 w-3 text-green-500" /> // 已完成
  } else if (task.startsAt) {
    // return <Icon.Cycle className="h-4 w-4 animate-spin text-orange-500" /> // 已经开始但还没结束
    return <Icon.FlashStroke className="h-3 w-3 text-orange-400" />
  } else {
    return <Icon.Clock className="h-3 w-3 text-neutral-900" />
  }
}

export default function Inspector({ data }: { data: ExtractExplorerItem<'FilePath'> | null }) {
  const inspector = useInspector()
  /**
   * listen to meta + I to toggle inspector
   * @todo 这个快捷键目前只是临时实现，之后应该统一的管理快捷键并且提供用户自定义的功能
   */
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.metaKey && e.key === 'i') {
        inspector.setShow(!inspector.show)
      }
    }
    window.addEventListener('keydown', handleKeyDown)
    return () => {
      window.removeEventListener('keydown', handleKeyDown)
    }
  }, [inspector])

  return inspector.show ? (
    <div className="border-app-line h-full w-64 overflow-auto border-l">
      {data ? (
        data.filePath.isDir ? (
          <FolderDetail data={data.filePath} />
        ) : data.assetObject ? (
          match(data)
            .with(matchExplorerItemWithType('video'), (props) => <VideoDetail {...props} />)
            .with(matchExplorerItemWithType('audio'), (props) => <AudioDetail {...props} />)
            .otherwise(() => <></>)
        ) : null
      ) : null}
    </div>
  ) : (
    <></>
  )
}

export function DetailTasks({ data }: { data: FilePath['assetObject'] }) {
  const { sortedTasks, handleJobsCancel } = useSortedTasks(data)

  return (
    <div className="text-xs">
      <div className="text-md mt-2 flex items-center justify-between">
        <div className="font-medium">Jobs</div>
        {sortedTasks.some((task) => task.exitCode === null) ? (
          <div className="text-ink/60 group flex items-center gap-1">
            <div className="px-1 opacity-0 transition-opacity duration-300 group-hover:opacity-100">
              Cancel pending jobs
            </div>
            <Button variant="ghost" size="xs" className="px-0" onClick={() => handleJobsCancel()}>
              <Icon.Close className="h-3 w-3" />
            </Button>
          </div>
        ) : null}
      </div>
      {sortedTasks.map((task) => (
        <div key={task.id} className="mt-2 flex items-center justify-between">
          <div className="text-ink/50">{task.taskType}</div>
          <TaskItemStatus task={task} />
        </div>
      ))}
    </div>
  )
}
