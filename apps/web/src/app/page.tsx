"use client";
import Link from "next/link";
import { useCallback, useEffect, useState, useContext } from "react";
import { rspc } from "@/lib/rspc";
import { CurrentLibrary } from "@/lib/library";
import { invoke } from "@tauri-apps/api/tauri";

const LibraryList: React.FC = () => {
  const { data: libraries, isLoading } = rspc.useQuery(["libraries.list"]);
  const libraryMut = rspc.useMutation("libraries.create");

  const createLibrary = useCallback(() => {
    libraryMut.mutate("a test library");
  }, [libraryMut]);

  const currentLibrary = useContext(CurrentLibrary);
  const handleLibraryClick = useCallback((libraryId: string) => {
    currentLibrary.setCurrentLibrary(libraryId);
  }, [currentLibrary]);

  return (
    <div className="bg-slate-400 px-4 py-8">
      <h1 className="my-2 font-bold text-xl">Libraries</h1>
      {libraries?.map((libraryId: string) => {
        return (
          <div key={libraryId} className="my-2">
            {/* <Link href={`/libraries/${libraryId}`}>{libraryId}</Link> */}
            <span
              onClick={() => handleLibraryClick(libraryId)}
              className="cursor-pointer text-blue-500 hover:underline"
            >{libraryId}</span>
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

export default function Home() {
  const doInvoke = async () => {
    /**
     * https://github.com/tauri-apps/tauri/discussions/5271#discussioncomment-3716246
     * This is caused by nextjs' SSR nature and an unfortunate design choice of the window module.
     * The workaround is to dynamically import the window module instead so that the code that
     * requires the navigator runs on the client side (btw all tauri apis only work on the
     * client side, but iirc only the window and path modules need dynamic imports).
     * 不能在上面直接 import '@tauri-apps/api'
     * 不然 @tauri-apps/api/helpers/os-check.js 会报错 navigator is not defined
     */
    // const { invoke } = await import('@tauri-apps/api');
    invoke("greet", { name: "World" }).then((response) => console.log(response));
  };

  let click = useCallback(async () => {
    await doInvoke();
  }, []);

  return (
    <main className="min-h-screen">
      <LibraryList />
      <div>
        <button className="w-24 h-24 bg-white" onClick={() => click()}>
          test
        </button>
      </div>
      <div>
        <Link href="/library" className="block p-2 bg-blue-400">
          go to library
        </Link>
      </div>
      <div>
        <Link href="/video-tasks" className="block p-2 bg-green-400">
          go to video-tasks
        </Link>
      </div>
      <div>
        <Link href="/search" className="block p-2 bg-green-200">
          go to search
        </Link>
      </div>
      <div className="bg-blue-500 p-4">
        <Link href="/files">
          <button className="p-4 text-white bg-black rounded-lg">
            direct to search test page
          </button>
        </Link>
      </div>
    </main>
  );
}
