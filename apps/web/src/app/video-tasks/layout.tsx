import AudioDialog from '@/app/video-tasks/_components/audio/dialog'
import VideoTaskHeader from '@/app/video-tasks/_components/header'
import { ReactNode } from 'react'

export default function VideoTaskLayout({ children }: { children: ReactNode }) {
  return (
    <main className="flex h-full flex-col">
      <VideoTaskHeader className="h-[54px]" />
      <div
        style={{
          height: `calc(100vh - 54px)`,
        }}
      >
        {children}
      </div>
      <AudioDialog />
    </main>
  )
}
