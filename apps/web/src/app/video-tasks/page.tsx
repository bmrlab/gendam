'use client'
import { ScrollArea } from '@muse/ui/v1/scroll-area'
import { useEffect, useRef } from 'react'
import FilterWidget from './_components/filter'
import TaskFooter from './_components/footer'
import VideoTasksList from './_components/task-list'
import useTaskList from './useTaskList'
import Viewport from '@/components/Viewport'
import VideoTaskHeader from './_components/header'
import AudioDialog from './_components/audio/AudioDialog'

export default function VideoTasksPage() {
  const { data: videos, isLoading, hasNextPage, fetchNextPage } = useTaskList({ limit: 10 })

  const bottomRef = useRef(null)

  useEffect(() => {
    // 防止初次加载
    const timer = setTimeout(() => {
      const observer = new IntersectionObserver(
        ([entry]) => {
          if (entry.isIntersecting) {
            if (hasNextPage) {
              fetchNextPage()
            }
          }
        },
        {
          root: null,
          threshold: 0.1,
        },
      )
      if (bottomRef.current) {
        observer.observe(bottomRef.current)
      }
    }, 500)

    return () => {
      clearTimeout(timer)
    }
  }, [fetchNextPage, hasNextPage])

  return (
    <Viewport.Page>
      <VideoTaskHeader />
      <Viewport.Content>
        <FilterWidget />
        <ScrollArea className="flex-1 rounded-[6px]">
          <VideoTasksList data={videos ?? []} isLoading={isLoading} />
          <div className="flex items-center justify-center p-4">
            {hasNextPage ? (
              <>
                <div className="text-xs text-slate-400">滚动加载更多</div>
                <div ref={bottomRef}></div>
              </>
            ) : (
              <div className="text-xs text-slate-400">没有更多了</div>
            )}
          </div>
        </ScrollArea>
      </Viewport.Content>
      <TaskFooter total={videos?.length ?? 0} />
      <AudioDialog />
    </Viewport.Page>
  )
}

// export default function Page() {
//   const videoTasklMut = rspc.useMutation("video.tasks.create");
//   let [videoPath, setVideoPath] = useState<string>("");
//   const videoPathInputRef = useRef<HTMLInputElement>(null);

//   const handleGetVideoFrames = useCallback((videoPath: string) => {
//     videoTasklMut.mutate(videoPath);
//   }, [videoTasklMut]);

//   const handleOpenFile = useCallback(async () => {
//     const selected = await selectFile();
//     if (selected) {
//       const videoPath = selected;
//       if (videoPathInputRef.current) {
//         videoPathInputRef.current.value = videoPath;
//       }
//       setVideoPath(videoPath);
//       videoTasklMut.mutate(videoPath);
//     }
//   }, [videoTasklMut]);

//   return (
//     <main className="min-h-screen p-12">
//       {/* <div>Path: {videoPath}</div> */}
//       <div className="">
//         <form onSubmit={(e: React.FormEvent<HTMLFormElement>) => {
//             e.preventDefault();
//             if (videoPathInputRef.current) {
//               let videoPath = videoPathInputRef.current.value;
//               setVideoPath(videoPath);
//               handleGetVideoFrames(videoPath);
//             }
//           }}
//           className="flex mb-4"
//         >
//           <input ref={videoPathInputRef} type="text" className="text-black block flex-1 px-4 py-2" />
//           <button className="ml-4 px-6 bg-black text-white" type="submit">get frames</button>
//           <button className="ml-4 px-6 bg-slate-800 text-white"
//             onClick={() => handleOpenFile()} type="button">选择文件</button>
//         </form>
//       </div>
//       <VideoTasksList></VideoTasksList>
//     </main>
//   );
// }
