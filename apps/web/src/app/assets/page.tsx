"use client";
import { useCallback, useEffect, useMemo, useState } from "react";
import Image from "next/image";
import { Folder_Light, Document_Light } from "@muse/assets/icons";
import { rspc } from "@/lib/rspc";
import type { FilePathQueryResult } from "@/lib/bindings";
import UploadButton from "@/components/UploadButton";
import styles from "./styles.module.css";

export default function Files() {
  // currentPath 必须以 / 结尾, 调用 setCurrentPath 的地方自行确保格式正确
  const [currentPath, setCurrentPath] = useState<string>("/");
  const { data: assets, isLoading, error } = rspc.useQuery(["assets.list", {
    path: currentPath,
    dirsOnly: false,
  }]);

  const createPathMut = rspc.useMutation(["assets.create_file_path"]);
  const createAssetMut = rspc.useMutation(["assets.create_asset_object"]);

  const goToDir = useCallback((dirName: string) => {
    let newPath = currentPath;
    if (dirName === "-1") {
      newPath = newPath.replace(/(.*\/)[^/]+\/$/, "$1");
    } else {
      newPath += dirName + "/";
    }
    setCurrentPath(newPath);
  }, [setCurrentPath, currentPath]);

  let handleDoubleClick = useCallback((asset: FilePathQueryResult/*(typeof assets)[number]*/) => {
    if (asset.isDir) {
      goToDir(asset.name);
    } else {
      //
    }
  }, [goToDir]);

  let handleCreateDir = useCallback(() => {
    let name = window.prompt("输入文件夹名称");
    if (!name) {
      return;
    }
    createPathMut.mutate({
      path: currentPath,
      name: name
    });
  }, [createPathMut, currentPath]);

  let [selectedId, setSelectedId] = useState<number|null>(null);

  let handleSelectFile = useCallback((fileFullPath: string) => {
    console.log("handleSelectFile", fileFullPath);
    createAssetMut.mutate({
      path: currentPath,
      localFullPath: fileFullPath
    })
  }, [createAssetMut, currentPath]);

  return (
    <div className="h-full flex flex-col">
      <div className="px-4 py-2 border-b border-slate-100 flex justify-between">
        <div className="flex items-center select-none">
          <div className="px-2 py-1">&lt;</div>
          <div className="px-2 py-1">&gt;</div>
          { currentPath !== "/" && (
            <div className="px-2 py-1 cursor-pointer" onClick={() => goToDir("-1")}>↑</div>
          )}
          <div className="ml-2 text-sm">{ currentPath === "/" ? "全部" : currentPath }</div>
        </div>
        <div className="flex items-center select-none">
          <div
            className="px-2 py-1 cursor-pointer text-sm"
            onClick={() => handleCreateDir()}
          >添加文件夹</div>
          <UploadButton onSelectFile={handleSelectFile}/>
        </div>
      </div>
      <div
        className="p-6 flex-1 flex flex-wrap items-start justify-start"
        onClick={() => setSelectedId(null)}
      >
        {assets && assets.map((asset) => (
          <div
            key={asset.id}
            className={
              `w-36 m-2 flex flex-col justify-between overflow-hidden cursor-default select-none
              ${selectedId === asset.id && styles["selected"]}`
            }
            onClick={(e) => {
              e.stopPropagation();
              setSelectedId(asset.id);
            }}
            onDoubleClick={(e) => {
              e.stopPropagation();
              setSelectedId(null);
              handleDoubleClick(asset);
            }}
          >
            <div className={`${styles["image"]} rounded-lg`}>
              {asset.isDir ? (
                <Image src={ Folder_Light } alt="folder"></Image>
              ) : (
                <Image src={ Document_Light } alt="folder"></Image>
              )}
            </div>
            <div className={`${styles["title"]} p-1 mt-1 mb-2 rounded-lg`}>
              <div
                className="leading-[1.4em] h-[2.8em] line-clamp-2 text-xs text-center"
              >{asset.name}</div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
