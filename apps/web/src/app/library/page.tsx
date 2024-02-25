"use client";
import { useCallback, useEffect, useState, useRef, useContext } from "react";
import { rspc } from "@/lib/rspc";
import Files from "./Files";

export default function Library() {
  const { data: homeDir } = rspc.useQuery(["files.home_dir"]);

  let [fullPath, setFullPath] = useState<string>("");

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
    if (homeDir) {
      setFullPath(homeDir);
    }
  }, [setFullPath, homeDir]);

  return (
    <main className="min-h-screen p-12">
      <div className="">{fullPath}</div>
      {fullPath && (
        <Files folderPath={fullPath} goToFolder={goToFolder} />
      )}
    </main>
  );
}
