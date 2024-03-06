"use client";
import Link from "next/link";
import Image from "next/image";
import { usePathname } from 'next/navigation'
import { useCallback, useEffect, useRef, useState, useContext } from "react";
import { rspc } from "@/lib/rspc";
import { CurrentLibrary } from "@/lib/library";
import { Muse_Logo, Chevron_Double } from "@muse/assets/svgs";

export default function Sidebar() {
  const panelRef = useRef<HTMLDivElement>(null);
  const [selectPanelOpen, setSelectPanelOpen] = useState(false);
  const { data: libraries } = rspc.useQuery(["libraries.list"]);
  const pathname = usePathname();
  const currentLibrary = useContext(CurrentLibrary);

  const switchLibrary = useCallback(async (libraryId: string) => {
    // console.log("switchLibrary");
    // currentLibrary.resetContext();
    await currentLibrary.setContext(libraryId);
  }, [currentLibrary]);

  useEffect(() => {
    function handleClickOutside(event: any) {
      // console.log(panelRef.current, event.target);
      if (panelRef.current && !panelRef.current.contains(event.target)) {
        setSelectPanelOpen(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => {
      document.removeEventListener("mousedown", handleClickOutside);
    };
  }, []);

  return (
    <div className="min-h-full w-60 bg-neutral-100 p-3">
      <div className="my-4 relative">
        <div className="flex items-center justify-start cursor-pointer"
          onClick={() => setSelectPanelOpen(true)}
        >
          <Image src={Muse_Logo} alt="Muse" className="w-8 h-8"></Image>
          <div className="mx-2 text-xs font-semibold w-32 overflow-hidden overflow-ellipsis whitespace-nowrap">
            Muse ({currentLibrary.id})
          </div>
          <Image src={Chevron_Double} alt="Chevron_Double" className="w-4 h-4"></Image>
        </div>
        {selectPanelOpen && (
          <div ref={panelRef} className="absolute z-10 left-32 top-3 w-60 p-1
              rounded-md bg-neutral-100 border border-neutral-200 shadow-sm">
            {libraries?.map((libraryId: string, index: number) => {
              return (
                <div key={libraryId}
                  className="px-3 py-2 flex items-center justify-start rounded-md
                    hover:bg-neutral-200 cursor-pointer"
                  onClick={() => switchLibrary(libraryId)}
                >
                  <Image src={Muse_Logo} alt="Muse" className="w-8 h-8"></Image>
                  <div className="mx-2 text-xs font-semibold w-48 overflow-hidden overflow-ellipsis whitespace-nowrap">
                    Muse ({libraryId})
                  </div>
                </div>
              );
            })}
          </div>
        )}
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
