"use client";
// import Link from "next/link";
import Image from "next/image";
import { useCallback, useEffect, useState, useContext } from "react";
import { rspc } from "@/lib/rspc";
import { CurrentLibrary } from "@/lib/library";
import { Muse_Logo, Chevron_Double } from "@muse/assets/svgs";

export default function LibrariesSelect() {
  const { data: libraries, isLoading } = rspc.useQuery(["libraries.list"]);
  const libraryMut = rspc.useMutation("libraries.create");

  const createLibrary = useCallback(() => {
    libraryMut.mutate("a test library");
  }, [libraryMut]);

  const currentLibrary = useContext(CurrentLibrary);
  const handleLibraryClick = useCallback(async (libraryId: string) => {
    await currentLibrary.setContext(libraryId);
  }, [currentLibrary]);

  return (
    <div className="bg-white w-screen h-screen flex flex-col items-center justify-center">
      <Image src={Muse_Logo} alt="Muse" className="w-8 h-8 mb-4"></Image>
      <div className="w-80 my-4 p-1 rounded-md bg-neutral-100 border border-neutral-200 shadow-sm">
        {(libraries??[]).length === 0 ? (
          <div className="px-3 py-2 text-xs text-center text-neutral-600">还未创建任何素材库，点击下方“创建”后继续</div>
        ) : (
          <div className="px-3 py-2 text-xs text-center text-neutral-600">选择素材库</div>
        )}
        {libraries?.map((libraryId: string, index: number) => {
          return (
            <div key={libraryId}
              className="px-3 py-2 flex items-center justify-start rounded-md
                hover:bg-neutral-200 cursor-pointer"
              onClick={() => handleLibraryClick(libraryId)}
            >
              <Image src={Muse_Logo} alt="Muse" className="w-8 h-8"></Image>
              <div className="mx-2 text-xs font-semibold w-64 overflow-hidden overflow-ellipsis whitespace-nowrap">
                Muse ({libraryId})
              </div>
            </div>
          );
        })}
        <div
          className="px-3 py-2 rounded-md hover:bg-neutral-200 cursor-pointer"
          onClick={() => createLibrary()}
        >
          <div className="text-sm text-center">+ 创建素材库</div>
        </div>
      </div>
    </div>
  )
}
