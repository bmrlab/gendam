"use client";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useRouter } from "next/navigation";
import Image from "next/image";
import { Folder_Light, Document_Light } from "@muse/assets/icons";
import { rspc } from "@/lib/rspc";
import type { FilePathQueryResult } from "@/lib/bindings";
import UploadButton from "@/components/UploadButton";
import { getLocalFileUrl } from "@/utils/file";
import styles from "./styles.module.css";

const TitleDialog: React.FC<{
  onConfirm: (title: string) => void,
  onCancel: () => void,
}> = ({ onConfirm, onCancel }) => {
  const inputRef = useRef<HTMLInputElement>(null);
  const handleSearch = useCallback((e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const keyword = inputRef.current?.value;
    if (!keyword) return;
    onConfirm(keyword);
  }, [onConfirm]);
  return (
    <div
      className="fixed z-20 left-0 top-0 w-full h-full bg-neutral-50/50 flex items-center justify-center"
      onClick={() => onCancel()}
    >
      <form
        className="block w-96 p-6 border bg-white/90 border-neutral-100 rounded-md shadow"
        onSubmit={handleSearch} onClick={(e) => e.stopPropagation()}
      >
        <div>创建文件夹</div>
        <input
          ref={inputRef} type="text"
          className="block w-full text-black bg-neutral-100 rounded-md my-4 px-4 py-2 text-sm"
          placeholder="搜索"
        />
        <button className="block w-full p-2 rounded-md text-sm text-center bg-blue-500 text-white" type="submit">确认</button>
      </form>
    </div>
  );
}

export default function Files() {
  const router = useRouter();
  // currentPath 必须以 / 结尾, 调用 setCurrentPath 的地方自行确保格式正确
  const [currentPath, setCurrentPath] = useState<string>("/");
  const { data: assets, isLoading, error } = rspc.useQuery(["assets.list", {
    path: currentPath,
    dirsOnly: false,
  }]);

  const revealMut = rspc.useMutation(["files.reveal"]);
  const createPathMut = rspc.useMutation(["assets.create_file_path"]);
  const createAssetMut = rspc.useMutation(["assets.create_asset_object"]);
  const processVideoMut = rspc.useMutation(["assets.process_video_asset"]);

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
    } else if (asset.assetObject) {
      // this will always be true if asset.isDir is false
      // revealMut.mutate("/" + asset.assetObject.id.toString());
      processVideoMut.mutate(asset.assetObject.id);
      router.push("/video-tasks");
    }
  }, [goToDir, processVideoMut, router]);

  let [selectedId, setSelectedId] = useState<number|null>(null);

  let handleSelectFile = useCallback((fileFullPath: string) => {
    createAssetMut.mutate({
      path: currentPath,
      localFullPath: fileFullPath
    })
  }, [createAssetMut, currentPath]);

  const [titleInputDialogVisible, setTitleInputDialogVisible] = useState(false);

  let handleCreateDir = useCallback(() => {
    setTitleInputDialogVisible(true);
  }, [setTitleInputDialogVisible]);

  const onConfirmTitleInput = useCallback((title: string) => {
    if (!title) {
      return;
    }
    createPathMut.mutate({
      path: currentPath,
      name: title,
    });
    setTitleInputDialogVisible(false);
  }, [createPathMut, currentPath]);

  const onCancelTitleInput = useCallback(() => {
    setTitleInputDialogVisible(false);
  }, [setTitleInputDialogVisible]);

  return (
    <div className="h-full flex flex-col">
      <div className="h-12 px-4 border-b border-neutral-100 flex justify-between">
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
        className="p-6 flex-1 flex flex-wrap content-start items-start justify-start"
        onClick={() => setSelectedId(null)}
      >
        {assets && assets.map((asset) => (
          <div
            key={asset.id}
            className={
              `m-2 flex flex-col items-center justify-start cursor-default select-none
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
            <div className={`${styles["image"]} w-32 h-32 rounded-lg overflow-hidden`}>
              {asset.isDir ? (
                <Image src={ Folder_Light } alt="folder"></Image>
              ) : (
                // <Image src={ Document_Light } alt="folder"></Image>
                <video controls={false} autoPlay muted loop style={{
                  width: "100%",
                  height: "100%",
                  objectFit: "cover",
                }}>
                  <source src={getLocalFileUrl(asset.assetObject?.localFullPath ?? "")} type="video/mp4" />
                </video>
              )}
            </div>
            <div className={`${styles["title"]} w-32 p-1 mt-1 mb-2 rounded-lg`}>
              <div
                className="leading-[1.4em] h-[2.8em] line-clamp-2 text-xs text-center"
              >{asset.name}</div>
            </div>
          </div>
        ))}
      </div>
      {titleInputDialogVisible && <TitleDialog onConfirm={onConfirmTitleInput} onCancel={onCancelTitleInput}/>}
    </div>
  );
}
