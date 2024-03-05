"use client";
import { useCallback, useEffect, useState, useContext } from "react";
import { open } from '@tauri-apps/api/dialog';

type Props = {
  onSelectFile: (fileFullPath: string) => void;
};

const TauriUploadButton: React.FC<Props> = ({ onSelectFile }) => {
  // TODO: remove `selectFile` in utils/file.ts
  let handleClick = useCallback(async () => {
    const result = await open({
      directory: false,
      multiple: false,
      filters: [{
        name: "Video",
        extensions: ["mp4", "mov", "avi", "mkv"],
      }]
    });
    // console.log("tauri selected file:", result);
    if (result) {
      const fileFullPath = result as string;
      onSelectFile(fileFullPath);
    } else {
      return null;
    }
  }, [onSelectFile]);

  return (
    <div>
      <form className="ml-4">
        <label
          htmlFor="file-input-select-new-asset"
          className="text-sm cursor-pointer"
          onClick={() => handleClick()}
        >上传文件</label>
      </form>
    </div>
  )
}

const WebUploadButton: React.FC = () => {
  let onFileInput = useCallback((e: React.FormEvent<HTMLInputElement>) => {
    console.log("form inpu selected file:", (e.target as any)?.files);
  }, []);

  return (
    <div>
      <form className="ml-4">
        <label
          htmlFor="file-input-select-new-asset"
          className="text-sm cursor-pointer"
        >上传文件</label>
        <input
          type="file" id="file-input-select-new-asset" className="hidden"
          onInput={onFileInput}
        />
        {/* <button type="submit">上传文件</button> */}
      </form>
    </div>
  )
}

export default function UploadButton({ onSelectFile }: Props) {
  if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
    return <TauriUploadButton onSelectFile={onSelectFile}/>;
  } else {
    return <WebUploadButton />
  }
}
