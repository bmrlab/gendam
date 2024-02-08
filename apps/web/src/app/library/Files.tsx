import { useCallback, useEffect, useState } from "react";
import type { File } from "./types";

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
            <span>{file.name}</span>
          </div>
        ))}
      </div>
    </div>
  );
}
