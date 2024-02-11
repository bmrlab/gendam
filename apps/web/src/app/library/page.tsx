"use client";
import { useCallback, useEffect, useState, useRef } from "react";
import Sidebar from "./Sidebar";
import Files from "./Files";

export default function Library() {
  let [fullPath, setFullPath] = useState<string>("/Users/xddotcom/Downloads");
  const fullPathInputRef = useRef<HTMLInputElement>(null);

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

  useEffect(() => {
    if (fullPathInputRef.current) {
      fullPathInputRef.current.value = fullPath;
    }
  }, [fullPath]);

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
        <Files folderPath={fullPath} goToFolder={goToFolder} />
      </div>
    </main>
  );
}
