"use client";
import { useCallback, useState, useRef, useMemo, FC } from "react";
import { rspc } from "@/lib/rspc";
import { getLocalFileUrl } from "@/utils/file";
import type { VideoTaskResult } from "@/lib/bindings";
// import { selectFile } from "@/utils/file";
import AudioDialog from "@/app/video-tasks/_compoents/audio/dialog";
import TaskContextMenu from "./_compoents/task-context-menu";

import { FileTypeEnum } from "@/app/video-tasks/_compoents/audio/export";
import MuseMultiSelect from "@/components/MultiSelect";
import { Button } from "@/components/ui/button";
import { useBoundStore } from "@/store";

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
  task: VideoItem["tasks"][number];
}> = ({ task }) => {
  const typeToName: { [key: string]: string } = {
    "Audio": "语音转译",
    // "Transcript": "语音转译",
    "TranscriptEmbedding": "语音转译",
    // "FrameCaption": "图像描述",
    "FrameCaptionEmbedding": "图像描述",
    // "Frame": "图像特征",
    "FrameContentEmbedding": "图像特征",
  };
  if (!typeToName[task.taskType]) {
    return <></>
  }
  if (!task.startsAt) {
    return (
      <div className="mr-2 px-3 py-1 bg-neutral-100/80 text-neutral-600 font-light text-xs rounded-full overflow-hidden overflow-ellipsis whitespace-nowrap"
      >{typeToName[task.taskType]}</div>
    );
  } else if (task.startsAt && !task.endsAt) {
    return (
      <div className="mr-2 px-3 py-1 bg-orange-100/80 text-orange-600 font-light text-xs rounded-full overflow-hidden overflow-ellipsis whitespace-nowrap"
      >{typeToName[task.taskType]}</div>
    );
  } else if (task.startsAt && task.endsAt) {
    return (
      <div className="mr-2 px-3 py-1 bg-green-100/80 text-green-600 font-light text-xs rounded-full overflow-hidden overflow-ellipsis whitespace-nowrap"
      >{typeToName[task.taskType]}</div>
    );
  } else {
    return <></>;
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
    return <div className="flex px-2 py-8 text-sm text-neutral-400 items-center justify-center">正在加载...</div>
  }

  return (
    <div className="p-4">
      {videos.map((video: VideoItem) => {
        return (
          <TaskContextMenu fileHash={video.videoFileHash} key={video.videoFileHash}>
            <div
              key={video.videoFileHash}
              className="flex justify-start py-3 px-5 border-b border-neutral-100 hover:bg-neutral-100"
            >
              <div
                className="w-16 h-16 bg-neutral-200 mr-4 flex items-center justify-center cursor-pointer"
                onClick={() => handleClickVideoFile(video)}
              >
                <video controls={false} autoPlay muted loop style={{
                  width: "100%", height: "100%", objectFit: "cover",
                }}>
                  <source src={getLocalFileUrl(video.videoPath)} type="video/mp4" />
                </video>
              </div>
              <div className="mb-2 break-words w-96">
                {/* {video.videoPath} ({video.videoFileHash}) */}
                <div className="mb-2 flex">
                  <div className="mr-3">MUSE 的视频</div>
                  <div className="w-32 overflow-hidden overflow-ellipsis whitespace-nowrap text-neutral-400 font-light text-sm">{video.videoPath}</div>
                </div>
                <div className="text-neutral-400 font-light text-sm flex">
                  <div>00:01:04</div>
                  <div className="mx-2">·</div>
                  <div>10.87 MB</div>
                  <div className="mx-2">·</div>
                  <div>1440 x 1080</div>
                </div>
              </div>
              <div className="flex flex-wrap items-end ml-auto">
                {video.tasks.map((task, index) => <VideoTaskStatus key={index} task={task} />)}
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

  const handleBatchExport = () => {
    setAudioDialogProps({ fileHash: [], title: '批量导出语音转译' })
    setAudioDialogOpen(true)
  }

  return (
    <main className="h-full flex flex-col">
      <div className="h-12 px-4 border-b border-neutral-100 flex justify-between">
        <div className="flex items-center select-none">
          <div className="px-2 py-1">&lt;</div>
          <div className="px-2 py-1">&gt;</div>
          <div className="ml-2 text-sm">任务列表</div>
        </div>
      </div>
      <VideoTasksList />
      <AudioDialog />
      <div className="w-[240px]">
        <MuseMultiSelect
          showValue
          placeholder="选择格式"
          options={Object.keys(FileTypeEnum).map((type) => ({
            label: FileTypeEnum[type as keyof typeof FileTypeEnum],
            value: type.toString(),
          }))}
        />
      </div>
      <Button className="mt-4" onClick={handleBatchExport}>
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
