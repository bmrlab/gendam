"use client";

import Image from "next/image";
import Link from "next/link";
import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/tauri";
import { createClient } from "@rspc/client";
// import { TauriTransport } from "@rspc/tauri";

type File = {
  name: string;
  is_dir: boolean;
};

export default function Home() {
  let [files, setFiles] = useState<File[]>([]);

  const doInvoke = async (subpath?: string): Promise<File[]> => {
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

    // const { createClient } = await import("@rspc/client");
    const { TauriTransport } = await import("@rspc/tauri");
    const client = createClient({
      transport: new TauriTransport(),
    });

    client.query(["version"]).then((data) => console.log("version", data));

    client.query(["users"]).then((data) => console.log("users", data));

    try {
      let files: File[] = subpath
        ? await client.query<string>(["files", subpath])
        : await client.query(["files"]);
      console.log(files);
      return files;
    } catch(err) {
      console.log(err);
      return [];
    }
  };

  // useEffect(() => {
  //   console.log('abc');
  //   doInvoke();
  // }, []);subpath
  let click = useCallback(
    async (subpath?: string) => {
      let _files: File[] = await doInvoke(subpath);
      setFiles(_files);
    },
    [setFiles]
  );

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
      <div className="flex flex-wrap">
        {files.map((file) => (
          <div
            key={file.name}
            className="w-36 h-36 border-2 border-neutral-800 m-2 text-xs flex flex-col justify-between"
          >
            <span>{file.name}</span>
            {file.is_dir ? (
              <button onClick={() => click(file.name)}>点击查看详情</button>
            ) : (
              <div></div>
            )}
          </div>
        ))}
      </div>

      <div className="bg-blue-500 py-96">
        <Link href="/files">
          <button className="p-4 text-white bg-black rounded-lg">
            direct to search test page
          </button>
        </Link>
      </div>
    </main>
  );
}
