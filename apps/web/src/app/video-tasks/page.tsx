"use client";
import { useCallback, useEffect, useState, useRef } from "react";
import { rspc } from "@/lib/rspc";

const VideoTasksList: React.FC = () => {
  const { data, isLoading, error } = rspc.useQuery(["video.list_video_tasks"]);
  // console.log(data);

  // useEffect(() => {
  //   //
  // }, []);

  if (isLoading) {
    return <div>Loading</div>
  }

  const status = (task: any) => {
    if (!task.startsAt) {
      return "未开始";
    } else if (task.startsAt && !task.endsAt) {
      return "进行中";
    } else if (task.startsAt && task.endsAt) {
      return "已完成"
    }
  }

  return (
    <div>
      {data.map((task: any) => {
        return (
          <div key={task.id} className="flex">
            <div className="mx-2">{ task.id }</div>
            <div className="mx-2">{ task.videoPath }</div>
            <div className="mx-2">{ task.videoFileHash }</div>
            <div className="mx-2">{ task.taskType }</div>
            <div className="mx-2">{status(task)}</div>
          </div>
        )
      })}
    </div>
  )
}

export default function Library() {
  // const videoTasklList = rspc.useQuery(["video.list_video_tasks"]);
  const videoTasklMut = rspc.useMutation("video.create_video_task");
  let [videoPath, setVideoPath] = useState<string>("");
  const videoPathInputRef = useRef<HTMLInputElement>(null);

  let handleGetVideoFrames = useCallback((videoPath: string) => {
    videoTasklMut.mutate(videoPath);
  }, [videoTasklMut]);

  return (
    <main className="min-h-screen p-12">
      <div>Path: {videoPath}</div>
      <div className="flex h-12">
        <input
          className="w-[800px] px-4 py-2"
          ref={videoPathInputRef}
        ></input>
        <button
          className="p-2 bg-slate-200 hover:bg-slate-400"
          onClick={() => {
            if (videoPathInputRef.current) {
              let videoPath = videoPathInputRef.current.value;
              setVideoPath(videoPath);
              handleGetVideoFrames(videoPath);
            }
          }}
        >get frames</button>
      </div>
      <VideoTasksList></VideoTasksList>
    </main>
  );
}
