import Link from "next/link";
import { useCallback, useEffect, useState } from "react";

export default function Sidebar() {
  return (
    <div className="min-h-full w-48 bg-slate-200">
      <div className="text-sm">
        <Link href="/library" className="block px-4 py-2 my-2 bg-slate-300">本地文件</Link>
        <Link href="/search" className="block px-4 py-2 my-2 bg-slate-300">搜索</Link>
        <Link href="/video-tasks" className="block px-4 py-2 my-2 bg-slate-300">视频任务</Link>
      </div>
    </div>
  );
}
