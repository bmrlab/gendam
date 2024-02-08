import { useCallback, useEffect, useState } from "react";
import type { File } from "./types";
import Image from "next/image";
import { Folder_Light } from "@muse/assets/icons";

type Props = {
  files: File[]
}

export default function Files({ files }: Props) {
  return (
    <div className="bg-blue-400">
      <div className="flex flex-wrap">
        {files.map((file) => (
          <div
            key={file.name}
            className="w-36 h-36 border-2 border-neutral-800 m-2 text-xs flex flex-col justify-between"
          >
            <Image src={Folder_Light} alt="folder"></Image>
            <span>{file.name}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
