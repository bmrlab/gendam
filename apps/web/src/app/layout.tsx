import { Inter } from 'next/font/google'
import '@muse/tailwind/globals.css'

const inter = Inter({ subsets: ['latin'] })

// import dynamic from 'next/dynamic';
// const ClientLayout = dynamic(() => import('@/components/ClientLayout'), {
//   loading: () => <div className="w-screen h-screen bg-white flex items-center justify-center">Loading...</div>,
//   ssr: false,
// });

import ClientLayout from '@/components/ClientLayout'
import Viewport from '@/components/Viewport'
import type { Metadata } from 'next'
export const metadata: Metadata = {
  title: 'Muse | a local DAM of videos',
  description: 'Muse is a local DAM for videos',
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <ClientLayout>
          <Viewport>
            <Viewport.Sidebar />
            {children}
            {/* children should be a Viewport.Page element */}
          </Viewport>
        </ClientLayout>
      </body>
    </html>
  )
}
