import { Inter } from 'next/font/google'
import '@gendam/tailwind/globals.css'

const inter = Inter({ subsets: ['latin'] })

// import dynamic from 'next/dynamic';
// const ClientLayout = dynamic(() => import('@/components/ClientLayout'), {
//   loading: () => <div className="w-screen h-screen bg-white flex items-center justify-center">Loading...</div>,
//   ssr: false,
// });

import ClientLayout from '@/components/ClientLayout'
import type { Metadata } from 'next'
export const metadata: Metadata = {
  title: 'GenDAM | A privacy first generative DAM.',
  description: 'A cross-platform desktop application for managing, processing, and searching multimedia content using Rust-based libraries and AI models.',
}

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>
        <ClientLayout>{children}</ClientLayout>
      </body>
    </html>
  )
}
