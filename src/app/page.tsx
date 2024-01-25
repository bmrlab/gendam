"use client"

import Image from "next/image";
// import { invoke } from '@tauri-apps/api';
import { useCallback, useEffect } from "react";

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
    const { invoke } = await import('@tauri-apps/api');
    invoke('greet', { name: 'World' }).then((response) => console.log(response));
    let files = await invoke('list_files');
    console.log(files);
  }

  // useEffect(() => {
  //   console.log('abc');
  //   doInvoke();
  // }, []);
  let click = useCallback(() => {
    doInvoke();
  }, []);

  return (
    <main className="flex min-h-screen">
      <button className="w-24 h-24 bg-white" onClick={click}>
        test
      </button>
    </main>
  );
}
