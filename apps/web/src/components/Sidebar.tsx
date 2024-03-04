"use client";
import Link from "next/link";
import { useCallback, useEffect, useState, useContext } from "react";
// import { rspc } from "@/lib/rspc";
import { CurrentLibrary } from "@/lib/library";

export default function Sidebar() {
  const currentLibrary = useContext(CurrentLibrary);

  const switchLibrary = useCallback(() => {
    console.log("switchLibrary");
    currentLibrary.resetContext();
  }, [currentLibrary]);

  return (
    <div className="min-h-full w-48 bg-slate-200">
      <div className="px-4 py-8">
        <h1 className="my-2 font-bold text-xl">Library:</h1>
        <div className="my-2">
          <div className="cursor-pointer text-blue-500 hover:underline
              overflow-hidden overflow-ellipsis whitespace-nowrap">{currentLibrary.id}</div>
        </div>
        <div onClick={() => switchLibrary()}
          className="cursor-pointer text-blue-500 hover:underline">switch</div>
      </div>
      <div className="text-sm">
        <Link href="/library" className="block px-4 py-2 my-2 bg-slate-300">本地文件(Test)</Link>
        <Link href="/assets" className="block px-4 py-2 my-2 bg-slate-300">素材库</Link>
        <Link href="/search" className="block px-4 py-2 my-2 bg-slate-300">搜索</Link>
        <Link href="/video-tasks" className="block px-4 py-2 my-2 bg-slate-300">视频任务</Link>
      </div>
    </div>
  );
}
