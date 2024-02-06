"use client";
import { useCallback, useEffect, useState, useRef } from "react";
import { createClient } from "@rspc/client";
import Sidebar from "./Sidebar";
import Files from "./Files";
import type { File } from "./types";

const getClient = async () => {
  const { TauriTransport } = await import("@rspc/tauri");
  const client = createClient({
    transport: new TauriTransport(),
  });
  return client;
}

export default function Library() {
  let [fullPath, setFullPath] = useState<string>("/Users/xddotcom/Downloads");
  let [files, setFiles] = useState<File[]>([]);
  const fullPathInputRef = useRef<HTMLInputElement>(null);

  const lsFiles = useCallback(async (fullPath: string) => {
    const client = await getClient();
    try {
      let files: File[] = await client.query<string>(["ls", fullPath]);
      console.log(files);
      setFiles(files);
    } catch(err) {
      console.log(err);
      setFiles([]);
    }
  }, []);

  useEffect(() => {
    if (fullPathInputRef.current) {
      fullPathInputRef.current.value = fullPath;
    }
    lsFiles(fullPath);
  }, [lsFiles, fullPath]);

  return (
    <main className="min-h-screen flex">
      <div className="min-h-screen">
        <Sidebar />
      </div>
      <div className="min-h-screen flex-1">
        <div className="flex">
          <input
            className="w-96"
            ref={fullPathInputRef}
          ></input>
          <button
            className="p-2 bg-slate-200 hover:bg-slate-400"
            onClick={() => {
              if (fullPathInputRef.current) {
                setFullPath(fullPathInputRef.current.value);
              }
            }}
          >ls</button>
        </div>
        <Files files={files} />
      </div>
    </main>
  );
}
