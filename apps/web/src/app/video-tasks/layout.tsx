import AudioDialog from './_components/audio/dialog'
import VideoTaskHeader from './_components/header'
import { ReactNode } from 'react'
import Viewport from '@/components/Viewport'

export default function VideoTaskLayout({ children }: { children: ReactNode }) {
  return (
    <Viewport.Page>
      <VideoTaskHeader className="h-[54px]" />
      <div
        style={{
          height: `calc(100vh - 54px)`,
        }}
      >
        {children}
      </div>
      <AudioDialog />
    </Viewport.Page>
  )
}
