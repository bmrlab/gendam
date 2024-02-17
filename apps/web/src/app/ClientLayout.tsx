"use client";
import { client, queryClient, rspc } from "@/lib/rspc";

export default function ClientLayout({
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
