"use client";
import { useCallback, useEffect, useState, useContext } from "react";
import { open } from '@tauri-apps/api/dialog';

type Props = {
  onSelectFiles: (fileFullPath: string[]) => void;
};

const TauriUploadButton: React.FC<Props> = ({ onSelectFiles }) => {
  let handleClick = useCallback(async () => {
    const results = await open({
      directory: false,
      multiple: true,
      filters: [{
        name: "Video",
        extensions: ["mp4", "mov", "avi", "mkv"],
      }]
    });
    console.log("tauri selected file:", results);
    if (results && results.length) {
      const fileFullPaths = results as string[];
      onSelectFiles(fileFullPaths);
    } else {
      return null;
    }
  }, [onSelectFiles]);

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

export default function UploadButton({ onSelectFiles }: Props) {
  if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
    return <TauriUploadButton onSelectFiles={onSelectFiles}/>;
  } else {
    return <WebUploadButton />
  }
}
