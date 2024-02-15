"use client";
import type { AppProps } from "next/app";
import { client, queryClient, rspc } from "@/lib/rspc";

export default function Layout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <rspc.Provider client={client} queryClient={queryClient}>
      <div>{children}</div>
    </rspc.Provider>
  );
}
