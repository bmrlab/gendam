'use client'
import type { VideoTaskResult } from '@/lib/bindings'
import { rspc } from '@/lib/rspc'
import { getLocalFileUrl } from '@/utils/file'
import { FC, useCallback, useMemo } from 'react'
// import { selectFile } from "@/utils/file";
import AudioDialog from '@/app/video-tasks/_compoents/audio/dialog'
import TaskContextMenu from './_compoents/task-context-menu'

import { AudioDialogEnum } from '@/app/video-tasks/store/audio-dialog'
import { Button } from '@/components/ui/button'
import { useBoundStore } from '@/store'

type VideoItem = {
  videoPath: string
  videoFileHash: string
  tasks: {
    taskType: string
    startsAt: string | null
    endsAt: string | null
  }[]
}

const VideoTaskStatus: React.FC<{
  task: VideoItem['tasks'][number]
}> = ({ task }) => {
  const typeToName: { [key: string]: string } = {
    Audio: '语音转译',
    // "Transcript": "语音转译",
    TranscriptEmbedding: '语音转译',
    // "FrameCaption": "图像描述",
    FrameCaptionEmbedding: '图像描述',
    // "Frame": "图像特征",
    FrameContentEmbedding: '图像特征',
  }
  if (!typeToName[task.taskType]) {
    return <></>
  }
  if (!task.startsAt) {
    return (
      <div className="mr-2 overflow-hidden overflow-ellipsis whitespace-nowrap rounded-full bg-neutral-100/80 px-3 py-1 text-xs font-light text-neutral-600">
        {typeToName[task.taskType]}
      </div>
    )
  } else if (task.startsAt && !task.endsAt) {
    return (
      <div className="mr-2 overflow-hidden overflow-ellipsis whitespace-nowrap rounded-full bg-orange-100/80 px-3 py-1 text-xs font-light text-orange-600">
        {typeToName[task.taskType]}
      </div>
    )
  } else if (task.startsAt && task.endsAt) {
    return (
      <div className="mr-2 overflow-hidden overflow-ellipsis whitespace-nowrap rounded-full bg-green-100/80 px-3 py-1 text-xs font-light text-green-600">
        {typeToName[task.taskType]}
      </div>
    )
  } else {
    return <></>
  }
}

const VideoTasksList: FC = () => {
  const { data, isLoading, error } = rspc.useQuery(['video.tasks.list'])
  const revealMut = rspc.useMutation('files.reveal')

  const videos = useMemo<VideoItem[]>(() => {
    if (isLoading) {
      return []
    }
    const groups: {
      [videoFileHash: string]: VideoItem
    } = {}
    data?.forEach((task: VideoTaskResult) => {
      if (!groups[task.videoFileHash]) {
        groups[task.videoFileHash] = {
          videoPath: task.videoPath,
          videoFileHash: task.videoFileHash,
          tasks: [],
        }
      }
      groups[task.videoFileHash].tasks.push({
        taskType: task.taskType,
        startsAt: task.startsAt,
        endsAt: task.endsAt,
      })
    })
    return Object.values(groups)
  }, [data, isLoading])

  let handleClickVideoFile = useCallback(
    (video: VideoItem) => {
      revealMut.mutate(video.videoPath)
    },
    [revealMut],
  )

  if (isLoading) {
    return <div className="flex items-center justify-center px-2 py-8 text-sm text-neutral-400">正在加载...</div>
  }

  return (
    <div className="p-4">
      {videos.map((video: VideoItem) => {
        return (
          <TaskContextMenu fileHash={video.videoFileHash} key={video.videoFileHash}>
            <div
              key={video.videoFileHash}
              className="flex justify-start border-b border-neutral-100 px-5 py-3 hover:bg-neutral-100"
            >
              <div
                className="mr-4 flex h-16 w-16 cursor-pointer items-center justify-center bg-neutral-200"
                onClick={() => handleClickVideoFile(video)}
              >
                <video
                  controls={false}
                  autoPlay
                  muted
                  loop
                  style={{
                    width: '100%',
                    height: '100%',
                    objectFit: 'cover',
                  }}
                >
                  <source src={getLocalFileUrl(video.videoPath)} type="video/mp4" />
                </video>
              </div>
              <div className="mb-2 w-96 break-words">
                {/* {video.videoPath} ({video.videoFileHash}) */}
                <div className="mb-2 flex">
                  <div className="mr-3">MUSE 的视频</div>
                  <div className="w-32 overflow-hidden overflow-ellipsis whitespace-nowrap text-sm font-light text-neutral-400">
                    {video.videoPath}
                  </div>
                </div>
                <div className="flex text-sm font-light text-neutral-400">
                  <div>00:01:04</div>
                  <div className="mx-2">·</div>
                  <div>10.87 MB</div>
                  <div className="mx-2">·</div>
                  <div>1440 x 1080</div>
                </div>
              </div>
              <div className="ml-auto flex flex-wrap items-end">
                {video.tasks.map((task, index) => (
                  <VideoTaskStatus key={index} task={task} />
                ))}
              </div>
            </div>
          </TaskContextMenu>
        )
      })}
    </div>
  )
}

export default function VideoTasksPage() {
  const setAudioDialogProps = useBoundStore.use.setAudioDialogProps()
  const setAudioDialogOpen = useBoundStore.use.setIsOpenAudioDialog()

  // TODO 换为真实数据
  const handleBatchExport = () => {
    setAudioDialogProps({
      type: AudioDialogEnum.batch,
      title: '批量导出语音转译',
      params: [
        {
          id: '71a4c82148fb9a991fd5ebd93699eddebd11b321c79e2b85929cf5aee9e071f1',
          label: '视频1号',
          image: 'https://placehold.co/100x200',
        },
        {
          id: 'bc9570687a9a2644baeac0be8b25f22d6b018d6e3d093403383a17e6ba594f6a',
          label: '视频2号',
          image: 'https://placehold.co/100x200',
        },
      ],
    })
    setAudioDialogOpen(true)
  }

  return (
    <main className="flex h-full flex-col">
      <div className="flex h-12 justify-between border-b border-neutral-100 px-4">
        <div className="flex select-none items-center">
          <div className="px-2 py-1">&lt;</div>
          <div className="px-2 py-1">&gt;</div>
          <div className="ml-2 text-sm">任务列表</div>
        </div>
      </div>
      <VideoTasksList />
      <AudioDialog />
      <Button className="mt-4 w-60" onClick={handleBatchExport}>
        批量音频导出
      </Button>
    </main>
  )
}

// export default function Page() {
//   const videoTasklMut = rspc.useMutation("video.tasks.create");
//   let [videoPath, setVideoPath] = useState<string>("");
//   const videoPathInputRef = useRef<HTMLInputElement>(null);

//   const handleGetVideoFrames = useCallback((videoPath: string) => {
//     videoTasklMut.mutate(videoPath);
//   }, [videoTasklMut]);

//   const handleOpenFile = useCallback(async () => {
//     const selected = await selectFile();
//     if (selected) {
//       const videoPath = selected;
//       if (videoPathInputRef.current) {
//         videoPathInputRef.current.value = videoPath;
//       }
//       setVideoPath(videoPath);
//       videoTasklMut.mutate(videoPath);
//     }
//   }, [videoTasklMut]);

//   return (
//     <main className="min-h-screen p-12">
//       {/* <div>Path: {videoPath}</div> */}
//       <div className="">
//         <form onSubmit={(e: React.FormEvent<HTMLFormElement>) => {
//             e.preventDefault();
//             if (videoPathInputRef.current) {
//               let videoPath = videoPathInputRef.current.value;
//               setVideoPath(videoPath);
//               handleGetVideoFrames(videoPath);
//             }
//           }}
//           className="flex mb-4"
//         >
//           <input ref={videoPathInputRef} type="text" className="text-black block flex-1 px-4 py-2" />
//           <button className="ml-4 px-6 bg-black text-white" type="submit">get frames</button>
//           <button className="ml-4 px-6 bg-slate-800 text-white"
//             onClick={() => handleOpenFile()} type="button">选择文件</button>
//         </form>
//       </div>
//       <VideoTasksList></VideoTasksList>
//     </main>
//   );
// }
