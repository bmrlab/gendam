"use client";
import Link from "next/link";
import { useCallback, useEffect, useState, useContext } from "react";
import { rspc } from "@/lib/rspc";
import { CurrentLibrary } from "@/lib/library";

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
    <div className="px-4 py-8">
      <h1 className="my-2 font-bold text-xl">Libraries</h1>
      {libraries?.map((libraryId: string) => {
        return (
          <div key={libraryId} className="my-2">
            {/* <Link href={`/libraries/${libraryId}`}>{libraryId}</Link> */}
            <div
              onClick={() => handleLibraryClick(libraryId)}
              className="cursor-pointer text-blue-500 hover:underline
                overflow-hidden overflow-ellipsis whitespace-nowrap"
            >{libraryId}</div>
          </div>
        );
      })}
      <div>
        <button className="px-4 py-2 bg-black text-white rounded-full"
          onClick={() => createLibrary()}>create</button>
      </div>
    </div>
  )
}
