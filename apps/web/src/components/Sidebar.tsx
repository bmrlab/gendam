"use client";
import Link from "next/link";
import { usePathname } from 'next/navigation'
import { useCallback, useEffect, useState, useContext } from "react";
// import { rspc } from "@/lib/rspc";
import { CurrentLibrary } from "@/lib/library";

export default function Sidebar() {
  const pathname = usePathname();
  const currentLibrary = useContext(CurrentLibrary);

  const switchLibrary = useCallback(() => {
    console.log("switchLibrary");
    currentLibrary.resetContext();
  }, [currentLibrary]);

  return (
    <div className="min-h-full w-60 bg-neutral-100 p-3">
      <div className="my-4">
        <h1 className="my-2 font-bold text-xl">Library:</h1>
        <div className="my-2">
          <div className="cursor-pointer text-blue-500 hover:underline
              overflow-hidden overflow-ellipsis whitespace-nowrap">{currentLibrary.id}</div>
        </div>
        <div onClick={() => switchLibrary()}
          className="cursor-pointer text-blue-500 hover:underline">switch</div>
      </div>
      <div className="text-sm">
        <Link href="/library"
          className={`block rounded-md px-4 py-2 mb-1 ${pathname === "/library" && "bg-neutral-200"}`}
        >本地文件(Test)</Link>
        <Link href="/assets"
          className={`block rounded-md px-4 py-2 mb-1 ${pathname === "/assets" && "bg-neutral-200"}`}
        >素材库</Link>
        <Link href="/search"
          className={`block rounded-md px-4 py-2 mb-1 ${pathname === "/search" && "bg-neutral-200"}`}
        >搜索</Link>
        <Link href="/video-tasks"
          className={`block rounded-md px-4 py-2 mb-1 ${pathname === "/video-tasks" && "bg-neutral-200"}`}
        >视频任务</Link>
      </div>
    </div>
  );
}
