// "use client";
import { Inter } from "next/font/google";
import "./globals.css";

const inter = Inter({ subsets: ["latin"] });

// import ClientLayout from "./ClientLayout";
// import dynamic from 'next/dynamic';
// const ClientLayout = dynamic(() => import('./ClientLayout'), {
//   loading: () => <div className="w-screen h-screen bg-white flex items-center justify-center">Loading...</div>,
//   ssr: false,
// });

import type { Metadata } from "next";
export const metadata: Metadata = {
  title: "Muse | a local DAM of videos",
  description: "Muse is a local DAM for videos",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className={inter.className}>{children}</body>
    </html>
  );
}
