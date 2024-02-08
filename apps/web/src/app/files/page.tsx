"use client";

import { useCallback, useState } from "react";

export default function Files() {
  const [path, setPath] = useState("");
  const [text, setText] = useState("");
  const [recordType, setRecordType] = useState("");

  const handleFile = useCallback(() => {
    import("@tauri-apps/api").then(({ invoke }) => {
      invoke<string>("handle_video_file", {
        videoPath: path,
      })
        .then((res) => {
          console.log(res);
        })
        .catch((err: any) => {
          console.error(err);
        });
    });
  }, [path]);

  const handleCaption = useCallback(() => {
    import("@tauri-apps/api").then(({ invoke }) => {
      invoke<string>("get_frame_caption", {
        videoPath: path,
      })
        .then((res) => {
          console.log(res);
        })
        .catch((err: any) => {
          console.error(err);
        });
    });
  }, [path]);

  const handleSearch = useCallback(() => {
    import("@tauri-apps/api").then(({ invoke }) => {
      invoke<string>("handle_search", {
        payload: {
          text,
          record_type:
            recordType.length > 0 &&
            ["Frame", "Transcript", "FrameCaption"].includes(recordType)
              ? recordType
              : undefined,
        },
      })
        .then((res) => {
          console.log(res);
        })
        .catch((err: any) => {
          console.error(err);
        });
    });
  }, [recordType, text]);

  return (
    <div className="bg-slate-200 p-12 flex flex-col space-y-24 min-h-screen">
      <div className="flex justify-start items-center space-x-12">
        <input
          className="p-2 border-black rounded-lg border text-black"
          onChange={(e) => {
            setPath(e.target.value);
          }}
        />
        <button className="bg-black p-4 text-white" onClick={handleFile}>
          generate artifacts
        </button>

        <button className="bg-black p-4 text-white" onClick={handleCaption}>
          generate captions
        </button>
      </div>

      <div className="flex justify-start items-center space-x-12">
        <input
          className="p-2 border-black rounded-lg border text-black"
          onChange={(e) => {
            setText(e.target.value);
          }}
        />
        <input
          className="p-2 border-black rounded-lg border text-black"
          onChange={(e) => {
            setRecordType(e.target.value);
          }}
          placeholder="valid record type: Frame, Transcript and FrameCaption, or leave it empty"
        />
        <button className="bg-black p-4 text-white" onClick={handleSearch}>
          search
        </button>
      </div>
    </div>
  );
}
