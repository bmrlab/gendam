'use client'
import UploadButton from '@/components/UploadButton'
import type { FilePathQueryResult } from '@/lib/bindings'
import { useCurrentLibrary } from '@/lib/library'
import { rspc } from '@/lib/rspc'
import { Document_Light, Folder_Light } from '@muse/assets/images'
import Image from 'next/image'
import { useRouter } from 'next/navigation'
import { useCallback, useRef, useState } from 'react'
import styles from './styles.module.css'

const TitleDialog: React.FC<{
  onConfirm: (title: string) => void
  onCancel: () => void
}> = ({ onConfirm, onCancel }) => {
  const inputRef = useRef<HTMLInputElement>(null)
  const handleSearch = useCallback(
    (e: React.FormEvent<HTMLFormElement>) => {
      e.preventDefault()
      const keyword = inputRef.current?.value
      if (!keyword) return
      onConfirm(keyword)
    },
    [onConfirm],
  )
  return (
    <div
      className="fixed left-0 top-0 z-20 flex h-full w-full items-center justify-center bg-neutral-50/50"
      onClick={() => onCancel()}
    >
      <form
        className="block w-96 rounded-md border border-neutral-100 bg-white/90 p-6 shadow"
        onSubmit={handleSearch}
        onClick={(e) => e.stopPropagation()}
      >
        <div>创建文件夹</div>
        <input
          ref={inputRef}
          type="text"
          className="my-4 block w-full rounded-md bg-neutral-100 px-4 py-2 text-sm text-black"
          placeholder="搜索"
        />
        <button className="block w-full rounded-md bg-blue-500 p-2 text-center text-sm text-white" type="submit">
          确认
        </button>
      </form>
    </div>
  )
}

export default function Files() {
  const currentLibrary = useCurrentLibrary()
  const router = useRouter()
  // currentPath 必须以 / 结尾, 调用 setCurrentPath 的地方自行确保格式正确
  const [currentPath, setCurrentPath] = useState<string>('/')
  const {
    data: assets,
    isLoading,
    error,
  } = rspc.useQuery([
    'assets.list',
    {
      path: currentPath,
      dirsOnly: false,
    },
  ])

  const revealMut = rspc.useMutation(['files.reveal'])
  const createPathMut = rspc.useMutation(['assets.create_file_path'])
  const createAssetMut = rspc.useMutation(['assets.create_asset_object'])
  // const processVideoMut = rspc.useMutation(["assets.process_video_asset"]);

  const goToDir = useCallback(
    (dirName: string) => {
      let newPath = currentPath
      if (dirName === '-1') {
        newPath = newPath.replace(/(.*\/)[^/]+\/$/, '$1')
      } else {
        newPath += dirName + '/'
      }
      setCurrentPath(newPath)
    },
    [setCurrentPath, currentPath],
  )

  let handleDoubleClick = useCallback(
    (asset: FilePathQueryResult /*(typeof assets)[number]*/) => {
      if (asset.isDir) {
        goToDir(asset.name)
      } else {
        if (asset.assetObject) {
          // this will always be true if asset.isDir is false
          revealMut.mutate('/' + asset.assetObject.id.toString())
        }
        // processVideoMut.mutate(asset.id);
        // router.push("/video-tasks");
      }
    },
    [goToDir, revealMut],
  )

  let [selectedId, setSelectedId] = useState<number | null>(null)

  let handleSelectFile = useCallback(
    (fileFullPath: string) => {
      createAssetMut.mutate({
        path: currentPath,
        localFullPath: fileFullPath,
      })
    },
    [createAssetMut, currentPath],
  )

  const [titleInputDialogVisible, setTitleInputDialogVisible] = useState(false)

  let handleCreateDir = useCallback(() => {
    setTitleInputDialogVisible(true)
  }, [setTitleInputDialogVisible])

  const onConfirmTitleInput = useCallback(
    (title: string) => {
      if (!title) {
        return
      }
      createPathMut.mutate({
        path: currentPath,
        name: title,
      })
      setTitleInputDialogVisible(false)
    },
    [createPathMut, currentPath],
  )

  const onCancelTitleInput = useCallback(() => {
    setTitleInputDialogVisible(false)
  }, [setTitleInputDialogVisible])

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-12 justify-between border-b border-neutral-100 px-4">
        <div className="flex select-none items-center">
          <div className="px-2 py-1">&lt;</div>
          <div className="px-2 py-1">&gt;</div>
          {currentPath !== '/' && (
            <div className="cursor-pointer px-2 py-1" onClick={() => goToDir('-1')}>
              ↑
            </div>
          )}
          <div className="ml-2 text-sm">{currentPath === '/' ? '全部' : currentPath}</div>
        </div>
        <div className="flex select-none items-center">
          <div className="cursor-pointer px-2 py-1 text-sm" onClick={() => handleCreateDir()}>
            添加文件夹
          </div>
          <UploadButton onSelectFile={handleSelectFile} />
        </div>
      </div>
      <div
        className="flex flex-1 flex-wrap content-start items-start justify-start p-6"
        onClick={() => setSelectedId(null)}
      >
        {assets &&
          assets.map((asset) => (
            <div
              key={asset.id}
              className={`m-2 flex cursor-default select-none flex-col items-center justify-start
              ${selectedId === asset.id && styles['selected']}`}
              onClick={(e) => {
                e.stopPropagation()
                setSelectedId(asset.id)
              }}
              onDoubleClick={(e) => {
                e.stopPropagation()
                setSelectedId(null)
                handleDoubleClick(asset)
              }}
            >
              <div className={`${styles['image']} h-32 w-32 overflow-hidden rounded-lg`}>
                {asset.isDir ? (
                  <Image src={Folder_Light} alt="folder" priority></Image>
                ) : asset.assetObject ? (
                  <video
                    controls={false}
                    autoPlay
                    muted
                    loop
                    style={{ width: '100%', height: '100%', objectFit: 'cover' }}
                  >
                    <source src={currentLibrary.getFileSrc(asset.assetObject.hash)} type="video/mp4" />
                  </video>
                ) : (
                  <Image src={Document_Light} alt="folder" priority></Image>
                )}
              </div>
              <div className={`${styles['title']} mb-2 mt-1 w-32 rounded-lg p-1`}>
                <div className="line-clamp-2 h-[2.8em] text-center text-xs leading-[1.4em]">{asset.name}</div>
              </div>
            </div>
          ))}
      </div>
      {titleInputDialogVisible && <TitleDialog onConfirm={onConfirmTitleInput} onCancel={onCancelTitleInput} />}
    </div>
  )
}
