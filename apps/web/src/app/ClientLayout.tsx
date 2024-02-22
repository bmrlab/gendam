"use client";
import { useCallback, useEffect, useState, useMemo } from "react";
import { client, createClientWithLibraryId, queryClient, rspc } from "@/lib/rspc";
import { CurrentLibrary } from "@/lib/library";

export default function ClientLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  let libraryIdInStorage: string | null = null;
  if (typeof window !== "undefined") {
    libraryIdInStorage = window.localStorage.getItem("current-library-id") ?? null;
    if (!libraryIdInStorage && window.location.pathname !== '/') {
      window.location.href = '/';
    }
  }

  const [currentLibraryId, setCurrentLibraryId] = useState<string|null>(libraryIdInStorage);

  const setCurrentLibrary = useCallback((id: string) => {
    setCurrentLibraryId(id);
    window.localStorage.setItem("current-library-id", id);
  }, [setCurrentLibraryId]);

  let client2 = useMemo(() => {
    return currentLibraryId ?
      createClientWithLibraryId(currentLibraryId) :
      client;
  }, [currentLibraryId]);

  return (
    <rspc.Provider client={client2} queryClient={queryClient}>
      <CurrentLibrary.Provider value={{
        id: currentLibraryId,
        setCurrentLibrary,
      }}>
        <>{children}</>
      </CurrentLibrary.Provider>
    </rspc.Provider>
  );
}
