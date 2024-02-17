"use client";
import { useCallback, useEffect, useState, useRef } from "react";
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
    <main className="min-h-screen p-12">
      <div className="">
        <form onSubmit={(e: React.FormEvent<HTMLFormElement>) => {
            e.preventDefault();
            if (fullPathInputRef.current) {
              setFullPath(fullPathInputRef.current.value);
            }
          }}
          className="flex mb-4"
        >
          <input ref={fullPathInputRef} type="text" className="block flex-1 px-4 py-2" />
          <button className="ml-4 px-6 bg-black text-white" type="submit">ls</button>
        </form>
      </div>
      <Files folderPath={fullPath} goToFolder={goToFolder} />
    </main>
  );
}
