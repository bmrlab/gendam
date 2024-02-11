"use client";
import { useCallback, useEffect, useState, useRef } from "react";
import { rspc } from "@/lib/rspc";

export default function Library() {
  // const videoFrameslMut = rspc.useMutation("video.create_video_frames");
  const videoFrameslMut = rspc.useMutation("video.create_video_task");
  let [videoPath, setVideoPath] = useState<string>("");
  const videoPathInputRef = useRef<HTMLInputElement>(null);

  let handleGetVideoFrames = useCallback((videoPath: string) => {
    videoFrameslMut.mutate(videoPath);
  }, [videoFrameslMut]);

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
    </main>
  );
}
