"use client";
// import type { AppProps } from "next/app";
import { client, queryClient, rspc } from "@/lib/rspc";
import Sidebar from "@/components/Sidebar";

export default function Layout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <main className="min-h-screen flex">
      <Sidebar />
      <rspc.Provider client={client} queryClient={queryClient}>
        <div className="flex-1">{children}</div>
      </rspc.Provider>
    </main>
  );
}
