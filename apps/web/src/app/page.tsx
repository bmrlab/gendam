"use client";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useCallback, useEffect, useState, useContext } from "react";
import { rspc, client } from "@/lib/rspc";
import { CurrentLibrary } from "@/lib/library";
import { invoke } from "@tauri-apps/api/tauri";

export default function HomePage() {
  const router = useRouter();
  useEffect(() => {
    router.push("/assets");
  }, [router]);
  return <></>;
}

function Home() {
  // const versionQuery = rspc.useQuery(["version"]);
  useEffect(() => {
    // console.log("versionQuery data", versionQuery.data);
    client.query(["version"], {
      // context: {
      //   headers: {
      //     "X-ABC": "1234567"
      //   }
      // }
    }).then((res: any) => {
      console.log("client query res", res)
    });
  }, []);

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
