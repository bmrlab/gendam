"use client"

import Image from "next/image";
import { useCallback, useEffect, useState } from "react";
// import { invoke } from '@tauri-apps/api';
// import { createClient } from "@rspc/client";
// import { TauriTransport } from "@rspc/tauri";

type File = {
  name: string;
  is_dir: boolean;
};

export default function Home() {
  let [files, setFiles] = useState<File[]>([]);

  const doInvoke = async (subpath?: string): Promise<File[]> => {
    const { createClient } = await import('@rspc/client');
    const { TauriTransport } = await import('@rspc/tauri');
    const client = createClient({
      transport: new TauriTransport(),
    });
    client.query(['version']).then((data) => console.log('version', data));

    /**
     * https://github.com/tauri-apps/tauri/discussions/5271#discussioncomment-3716246
     * This is caused by nextjs' SSR nature and an unfortunate design choice of the window module.
     * The workaround is to dynamically import the window module instead so that the code that
     * requires the navigator runs on the client side (btw all tauri apis only work on the
     * client side, but iirc only the window and path modules need dynamic imports).
     * 不能在上面直接 import '@tauri-apps/api'
     * 不然 @tauri-apps/api/helpers/os-check.js 会报错 navigator is not defined
     */
    const { invoke } = await import('@tauri-apps/api');
    invoke('greet', { name: 'World' }).then((response) => console.log(response));
    invoke('list_users').then((response) => console.log('users', response));

    let files: File[] = subpath ?
      await invoke('list_files', { subpath: subpath }) :
      await invoke('list_files');
    console.log(files);
    return files;
  }

  // useEffect(() => {
  //   console.log('abc');
  //   doInvoke();
  // }, []);subpath
  let click = useCallback(async (subpath?: string) => {
    let _files: File[] = await doInvoke(subpath);
    setFiles(_files);
  }, [setFiles]);

  return (
    <main className="min-h-screen">
      <div>
        <button className="w-24 h-24 bg-white" onClick={() => click()}>
          test
        </button>
      </div>
      <div className="flex flex-wrap">
        {files.map((file) => (
          <div key={file.name} className="w-36 h-36 border-2 border-neutral-800 m-2 text-xs flex flex-col justify-between">
            <span>{file.name}</span>
            {file.is_dir ? (
              <button onClick={() => click(file.name)}>点击查看详情</button>
            ) : <div></div>}
          </div>
        ))}
      </div>
    </main>
  );
}
