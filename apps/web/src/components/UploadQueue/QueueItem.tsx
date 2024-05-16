'use client'
import { ExplorerItem } from '@/Explorer/types'
import { type FileItem } from '@/components/UploadQueue/store'
import { Video_File } from '@gendam/assets/images'
import classNames from 'classnames'
import Image from 'next/image'
import { HTMLAttributes, PropsWithChildren } from 'react'
// import { twx } from '@/lib/utils'
// const QueueItem = twx.div`flex items-center justify-start pl-2 pr-4 py-2`

const QueueItem = ({
  file,
  children,
  icon,
  status,
  className,
  ...props
}: PropsWithChildren<{
  file: FileItem | ExplorerItem
  icon?: React.ReactNode
  status?: React.ReactNode
}> &
  HTMLAttributes<HTMLDivElement>) => {
  // const splits = file.localFullPath.split('/')
  // const fileName = splits.length > 0 ? splits[splits.length - 1] : file.localFullPath
  const fileName = file.name
  return (
    <div
      {...props}
      className={classNames(
        'border-app-line group flex items-center justify-start gap-1 border-b px-3 py-2',
        className,
      )}
    >
      <Image src={Video_File} alt="document" className="h-6 w-6" priority></Image>
      <div className="mx-1 flex-1 overflow-hidden text-xs">
        <div className="mb-1 truncate">{fileName}</div>
        {status}
      </div>
      {icon}
      {/* <div className="ml-auto">{children}</div> */}
    </div>
  )
}

export default QueueItem
