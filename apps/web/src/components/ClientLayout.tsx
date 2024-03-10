"use client";
import { useCallback, useEffect, useState, useMemo } from "react";
import { client, queryClient, rspc } from "@/lib/rspc";
import { CurrentLibrary, CurrentLibraryStorage } from "@/lib/library";
import LibrariesSelect from "@/components/LibrariesSelect";

export default function ClientLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const [pending, setPending] = useState(true);
  const [libraryId, setLibraryId] = useState<string|null>(null);

  useEffect(() => {
    CurrentLibraryStorage.get().then((libraryIdInStorage) => {
      setLibraryId(libraryIdInStorage);
      setPending(false);
    }).catch(error => {
      console.log('CurrentLibraryStorage.get() error:', error);
      setLibraryId(null);
      setPending(false);
    })
  }, [setLibraryId, setPending]);

  const setContext = useCallback(async (id: string) => {
    setLibraryId(id);
    setPending(true);
    await CurrentLibraryStorage.set(id)
    setPending(false);
    // 最后 reload 一下，用新的 library 请求数据过程中，页面上还残留着上个 library 已请求的数据
    location.reload();
  }, [setLibraryId]);

  return pending ? (<></>) : (
    <CurrentLibrary.Provider value={{
      id: libraryId,
      setContext,
    }}>
      <rspc.Provider client={client} queryClient={queryClient}>
        {libraryId ? (
          <>{children}</>
        ) : (
          <LibrariesSelect />
        )}
      </rspc.Provider>
    </CurrentLibrary.Provider>
  );
}
