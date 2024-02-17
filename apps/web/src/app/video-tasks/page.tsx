"use client";
import { useCallback, useEffect, useState, useRef, useMemo } from "react";
import { rspc } from "@/lib/rspc";
import { getContentUrl } from "@/utils/file";
import type { VideoTaskResult } from "@/lib/bindings";

type VideoItem = {
  videoPath: string;
  videoFileHash: string;
  tasks: {
    taskType: string;
    startsAt: string | null;
    endsAt: string | null;
  }[];
}

const status = (task: {
  startsAt: string | null;
  endsAt: string | null;
}) => {
  if (!task.startsAt) {
    return ["⚪️", "未开始"];
  } else if (task.startsAt && !task.endsAt) {
    return ["🟠", "进行中"];
  } else if (task.startsAt && task.endsAt) {
    return ["🟢", "已完成"]
  } else {
    return ["", ""];
  }
}

const VideoTasksList: React.FC = () => {
  const { data, isLoading, error } = rspc.useQuery(["video.tasks.list"]);
  const revealMut = rspc.useMutation("files.reveal");

  const videos = useMemo<VideoItem[]>(() => {
    if (isLoading) {
      return [];
    }
    const groups: {
      [videoFileHash: string]: VideoItem;
    } = {};
    data?.forEach((task: VideoTaskResult) => {
      if (!groups[task.videoFileHash]) {
        groups[task.videoFileHash] = {
          videoPath: task.videoPath,
          videoFileHash: task.videoFileHash,
          tasks: []
        };
      }
      groups[task.videoFileHash].tasks.push({
        taskType: task.taskType,
        startsAt: task.startsAt,
        endsAt: task.endsAt
      });
    });
    return Object.values(groups);
  }, [data, isLoading]);

  let handleClickVideoFile = useCallback((video: VideoItem) => {
    revealMut.mutate(video.videoPath);
  }, [revealMut]);

  if (isLoading) {
    return <div>Loading</div>
  }

  return (
    <div>
      {videos.map((video: VideoItem) => {
        return (
          <div key={video.videoFileHash} className="flex my-4">
            <div
              className="w-16 h-16 bg-slate-200 mr-2 flex items-center justify-center cursor-pointer"
              onClick={() => handleClickVideoFile(video)}
            >
              <video style={{ width: "100%", height: "auto" }}>
                <source src={getContentUrl(video.videoPath)} type="video/mp4" />
              </video>
            </div>
            <div className="p-1">
              <div className="text-xs mb-2">{video.videoPath} ({video.videoFileHash})</div>
              <div className="flex">
                {video.tasks.map((task, index) => {
                  let [icon, text] = status(task);
                  return (
                    <div key={index} className="mr-2 px-3 py-1 bg-slate-200 rounded-lg overflow-hidden text-xs">
                      <span className="mr-1">{icon}</span>
                      <span>{task.taskType}</span>
                    </div>
                  )
                })}
              </div>
            </div>
            {/* <div className="mx-2">{ task.id }</div>
            <div className="mx-2">{ task.videoPath }</div>
            <div className="mx-2">{ task.videoFileHash }</div>
            <div className="mx-2">{ task.taskType }</div>
            <div className="mx-2">}</div> */}
          </div>
        )
      })}
    </div>
  )
}

export default function VideoTasks() {
  // const videoTasklList = rspc.useQuery(["video.list_video_tasks"]);
  const videoTasklMut = rspc.useMutation("video.tasks.create");
  let [videoPath, setVideoPath] = useState<string>("");
  const videoPathInputRef = useRef<HTMLInputElement>(null);

  let handleGetVideoFrames = useCallback((videoPath: string) => {
    videoTasklMut.mutate(videoPath);
  }, [videoTasklMut]);

  return (
    <main className="min-h-screen p-12">
      {/* <div>Path: {videoPath}</div> */}
      <div className="">
        <form onSubmit={(e: React.FormEvent<HTMLFormElement>) => {
            e.preventDefault();
            if (videoPathInputRef.current) {
              let videoPath = videoPathInputRef.current.value;
              setVideoPath(videoPath);
              handleGetVideoFrames(videoPath);
            }
          }}
          className="flex mb-4"
        >
          <input ref={videoPathInputRef} type="text" className="block flex-1 px-4 py-2" />
          <button className="ml-4 px-6 bg-black text-white" type="submit">get frames</button>
        </form>
      </div>
      <VideoTasksList></VideoTasksList>
    </main>
  );
}
