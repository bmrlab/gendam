'use client'
import TaskFooter from '@/app/video-tasks/_components/footer'
import VideoTasksList from '@/app/video-tasks/_components/task-list'
import { ScrollArea } from '@/components/ui/scroll-area'
import { rspc } from '@/lib/rspc'
// import { useMemo } from 'react'
// import type { VideoItem } from '@/app/video-tasks/_components/task-item'

export default function VideoTasksPage() {
  const {
    data: videosWithTasks,
    isLoading, error
  } = rspc.useQuery(['video.tasks.list']);

  return (
    <div className="flex h-full flex-col">
      <ScrollArea className="flex-1 rounded-[6px]">
        <VideoTasksList data={videosWithTasks ?? []} isLoading={isLoading} />
      </ScrollArea>
      <TaskFooter total={videosWithTasks?.length ?? 0} />
    </div>
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
