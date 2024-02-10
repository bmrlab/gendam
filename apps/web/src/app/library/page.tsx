"use client";
import { useCallback, useEffect, useState, useRef } from "react";
// import { createClient } from "@rspc/client";
import { httpLink, initRspc } from "@rspc/client";
import Sidebar from "./Sidebar";
import Files from "./Files";
import type { File } from "./types";
import type { Procedures } from "@/lib/bindings";

const getClient = async () => {
  const links = [];
  if (typeof window.__TAURI__ !== 'undefined') {
    const { tauriLink } = await import("@rspc/tauri");
    links.push(tauriLink());
  } else {
    links.push(httpLink({
      url: "http://localhost:3001/rspc",
    }));
  }
  const client = initRspc<Procedures>({ links });
  return client;
}

export default function Library() {
  let [fullPath, setFullPath] = useState<string>("/Users/xddotcom/Downloads");
  let [files, setFiles] = useState<File[]>([]);
  const fullPathInputRef = useRef<HTMLInputElement>(null);

  const lsFiles = useCallback(async (fullPath: string) => {
    const client = await getClient();
    // client.query(["version"]).then((data) => console.log("!!data!!", data)).catch(err => {
    //   console.log("version err", err);
    // });
    try {
      let files: File[] = await client.query(["ls", fullPath]);
      // console.log(files);
      setFiles(files);
    } catch(err) {
      // console.log(err);
      setFiles([]);
    }
  }, []);

  const goToFolder = useCallback((folderName: string) => {
    let newFullPath = fullPath + (fullPath.endsWith("/") ? "" : "/");
    if (folderName === "-1") {
      newFullPath = newFullPath.replace(/(.*\/)[^/]+\/$/, "$1");
    } else {
      newFullPath += folderName;
    }
    // console.log("goto", folderName);
    setFullPath(newFullPath);
  }, [setFullPath, fullPath]);

  const revealFile = useCallback(async (fileName: string) => {
    const client = await getClient();
    let newFullPath = fullPath + (fullPath.endsWith("/") ? "" : "/");
    newFullPath += fileName;
    let result = await client.mutation(["reveal", newFullPath]);
  }, [fullPath]);

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
        <Files files={files} goToFolder={goToFolder} revealFile={revealFile} />
      </div>
    </main>
  );
}
