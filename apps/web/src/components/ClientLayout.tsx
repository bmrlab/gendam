"use client";
import { useCallback, useEffect, useState, useMemo } from "react";
import { client, queryClient, rspc } from "@/lib/rspc";
import { CurrentLibrary } from "@/lib/library";
import LibrariesSelect from "@/components/LibrariesSelect";

export default function ClientLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const [pending, setPending] = useState(true);
  const [libraryId, setLibraryId] = useState<string|null>(null);

  useEffect(() => {
    client.query(["libraries.get_current_library"]).then((libraryIdInStorage) => {
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
    try {
      await client.mutation(["libraries.set_current_library", id]);
      // setPending(false);
      // 最后 reload 一下，用新的 library 请求数据过程中，页面上还残留着上个 library 已请求的数据
      // 既然要 reload，就不设置 setPending(false) 了
      location.reload();
    } catch(err) {
      console.error('CurrentLibraryStorage.set() error:', err);
    }
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
