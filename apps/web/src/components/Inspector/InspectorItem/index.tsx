'use client'

import { useInspector } from '@/components/Inspector/store'
import { matchExplorerItemWithType } from '@/Explorer/pattern'
import { ExplorerItem, RawFilePath } from '@/Explorer/types'
import { AssetObject, FileHandlerTask } from '@/lib/bindings'
import { formatBytes, formatDateTime } from '@/lib/utils'
import Icon from '@gendam/ui/icons'
import { Button } from '@gendam/ui/v2/button'
import { PropsWithChildren, ReactElement, useEffect, useRef } from 'react'
import { match } from 'ts-pattern'
import AudioDetail from './Audio'
import FolderDetail from './Folder'
import { useSortedTasks } from './hooks'
import ImageDetail from './Image'
import RawTextDetail from './RawText'
import VideoDetail from './Video'
import WebPageDetail from './WebPage'

export default function InspectorItem({ data }: { data: ExplorerItem | null }) {
  const containerRef = useRef<HTMLDivElement>(null)
  const { setViewerHeight } = useInspector()

  useEffect(() => {
    if (!containerRef.current) return

    setViewerHeight(containerRef.current.clientWidth)
  }, [containerRef, containerRef.current?.clientWidth, setViewerHeight])

  return (
    <div className="h-full w-full overflow-x-hidden overflow-y-scroll" ref={containerRef}>
      {data?.type === 'FilePathDir' ? (
        <FolderDetail data={data.filePath} />
      ) : data?.type === 'FilePathWithAssetObject' ? (
        match(data)
          .with(matchExplorerItemWithType('video'), (props) => <VideoDetail {...props} />)
          .with(matchExplorerItemWithType('audio'), (props) => <AudioDetail {...props} />)
          .with(matchExplorerItemWithType('image'), (props) => <ImageDetail {...props} />)
          .with(matchExplorerItemWithType('rawText'), (props) => <RawTextDetail {...props} />)
          .with(matchExplorerItemWithType('webPage'), (props) => <WebPageDetail {...props} />)
          .otherwise(() => <></>)
      ) : (
        <></>
      )}
    </div>
  )
}

export function InspectorItemContainer({ children }: PropsWithChildren) {
  return (
    <div className="p-3">
      {children}
      {/* blank area at the bottom */}
      <div className="mt-6"></div>
    </div>
  )
}

export function InspectorItemViewer({ children }: PropsWithChildren) {
  const { viewerHeight } = useInspector()

  return (
    <div
      className="bg-app-overlay/50 relative overflow-hidden"
      style={{
        height: `${viewerHeight}px`,
      }}
    >
      {children}
    </div>
  )
}

export function InspectorItemFilePath({ filePath }: { filePath: RawFilePath }) {
  return (
    <div className="mt-3 overflow-hidden">
      <div className="text-ink line-clamp-2 break-all text-sm font-medium">{filePath.name}</div>
      <div className="text-ink/50 mt-1 line-clamp-2 text-xs">Location {filePath.materializedPath}</div>
    </div>
  )
}

export function InspectorItemDivider() {
  return <div className="bg-app-line mb-3 mt-3 h-px" />
}

export function InspectorItemMetadataItem({ name, children }: PropsWithChildren<{ name: string }>) {
  return (
    <div className="mt-2 flex justify-between gap-2">
      <div className="text-ink/50 capitalize">{name}</div>
      <div className="truncate">{children}</div>
    </div>
  )
}

export function InspectorItemMetadata<T extends AssetObject>({
  data,
  children,
}: {
  data: T
  children?: (data: T) => ReactElement
}) {
  return (
    <div className="text-xs">
      <div className="text-md font-medium">Information</div>
      <InspectorItemMetadataItem name="Size">{formatBytes(data.size)}</InspectorItemMetadataItem>
      <InspectorItemMetadataItem name="Type">{data.mimeType}</InspectorItemMetadataItem>

      {children?.(data)}

      <InspectorItemMetadataItem name="Created">{formatDateTime(data.createdAt)}</InspectorItemMetadataItem>
      <InspectorItemMetadataItem name="Modified">{formatDateTime(data.updatedAt)}</InspectorItemMetadataItem>
    </div>
  )
}

function TaskItemStatus({ task }: { task: FileHandlerTask }) {
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

export function InspectorItemTasks({ sortedTasks, handleJobsCancel }: ReturnType<typeof useSortedTasks>) {
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
