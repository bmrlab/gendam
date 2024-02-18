"use client";
import Image from "next/image";
import { useCallback, useEffect, useState, useRef } from "react";
import { rspc, client } from "@/lib/rspc";
import { getLocalFileUrl } from "@/utils/file";
import type { SearchResultPayload } from "@/lib/bindings";

type VideoItem = {
  videoSrc: string;
  startTime: number;
};

const VideoPreview: React.FC<{ videoItem: VideoItem }> = ({ videoItem }) => {
  const videoRef = useRef<HTMLVideoElement>(null);
  const { videoSrc } = videoItem;
  let startTime = Math.max(0, (videoItem.startTime / 1e3) - 0.5);
  let endTime = startTime + 2;

  useEffect(() => {
    const video = videoRef.current;
    if (!video) return;
    video.currentTime = startTime;
    video.ontimeupdate = () => {
      if (video.currentTime >= endTime) {
        video.pause();
        video.ontimeupdate = null;
      }
    };
  }, [startTime, endTime]);

  return (
    <video ref={videoRef} controls autoPlay style={{
      width: "100%",
      height: "auto",
    }}>
      <source src={videoSrc} type="video/mp4" />
      Your browser does not support the video tag.
    </video>
  );
}

export default function Search() {
  const [searchKeyword, setSearchKeyword] = useState("");
  const queryRes = rspc.useQuery(["video.search.all", searchKeyword]);
  // cosnt { data, isLoading, error } = queryRes;
  const searchInputRef = useRef<HTMLInputElement>(null);
  const [videoItem, setVideoItem] = useState<VideoItem | null>(null);

  const handleVideoClick = useCallback((item: SearchResultPayload) => {
    setVideoItem({
      videoSrc: getLocalFileUrl(item.videoPath),
      startTime: item.startTime,
    });
  }, [setVideoItem]);

  const handleSearch = useCallback((e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const keyword = searchInputRef.current?.value;
    if (!keyword) return;
    setSearchKeyword(keyword);
  }, [setSearchKeyword]);

  return (
    <main className="min-h-screen p-12">
      <div>
        <form onSubmit={handleSearch} className="flex mb-4">
          <input ref={searchInputRef} type="text" className="block flex-1 px-4 py-2" />
          <button className="ml-4 px-6 bg-black text-white" type="submit">Search</button>
        </form>
      </div>
      {queryRes.isLoading ? (
        <div className="flex px-2 py-8 items-center justify-center">正在搜索...</div>
      ) : (
        <div className="flex flex-wrap">
          {queryRes.data?.map((item: SearchResultPayload) => {
            return (
              <div key={item.imagePath} className="m-4">
                <div className="relative w-64 h-36">
                  <Image
                    fill={true} style={{ objectFit: "cover" }}
                    src={getLocalFileUrl(item.imagePath)}
                    alt={item.imagePath}
                  ></Image>
                </div>
                <div className="cursor-pointer text-center" onClick={() => handleVideoClick(item)}>查看</div>
              </div>
            )
          })}
        </div>
      )}
      {videoItem && (
        <div className="fixed left-0 top-0 w-full h-full flex items-center justify-center">
          <div className="bg-black absolute left-0 top-0 w-full h-full opacity-70"
            onClick={() => setVideoItem(null)}></div>
          <div className="relative w-[80%]">
            <VideoPreview videoItem={videoItem}></VideoPreview>
          </div>
        </div>
      )}
    </main>
  );
}
