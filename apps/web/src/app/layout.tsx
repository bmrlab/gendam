import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import dynamic from 'next/dynamic';
import { client, queryClient, rspc } from "@/lib/rspc";

const ClientLayout = dynamic(() => import('./ClientLayout'), {
  loading: () => <div className="w-screen h-screen bg-white flex items-center justify-center">Loading...</div>,
  ssr: false,
})

const inter = Inter({ subsets: ["latin"] });

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
      <body className={inter.className}>
        <ClientLayout>{children}</ClientLayout>
      </body>
    </html>
  );
}
