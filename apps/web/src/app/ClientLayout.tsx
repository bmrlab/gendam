"use client";
// import { useRouter } from 'next/navigation'
import { usePathname } from 'next/navigation'
import { useCallback, useEffect, useState, useMemo } from "react";
import { client, createClientWithLibraryId, queryClient, rspc } from "@/lib/rspc";
import { CurrentLibrary, CurrentLibraryStorage } from "@/lib/library";
import LibrariesSelect from "@/components/LibrariesSelect";

export default function ClientLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  // const router = useRouter();
  // const pathname = usePathname();

  const [pending, setPending] = useState(true);
  const [libraryId, setLibraryId] = useState<string|null>(null);

  useEffect(() => {
    CurrentLibraryStorage.get().then((libraryIdInStorage) => {
      // if (!libraryIdInStorage && pathname !== '/') {
      //   window.location.href = '/';
      //   // router.push('/library/new')
      // }
      setLibraryId(libraryIdInStorage);
      setPending(false);
    })
  }, [setLibraryId]);

  const setContext = useCallback(async (id: string) => {
    setLibraryId(id);
    await CurrentLibraryStorage.set(id)
    setPending(true);
    /**
     * TODO: router.refresh 看起来会缓存 fetch 请求参数, 导致 rspc.useQuery 里 libraryId 没切换
     * 除非是切换到其他 tab 再切换回来才会更新, 这样就有问题, 展示改成刷新整个页面
     */
    // router.refresh();
    location.reload();
  }, [setLibraryId]);

  const resetContext = useCallback(async () => {
    setLibraryId(null);
    await CurrentLibraryStorage.reset();
    // location.reload();
  }, [setLibraryId]);

  let client2 = useMemo(() => {
    return libraryId ?
      createClientWithLibraryId(libraryId) :
      client;
  }, [libraryId]);

  return pending ? (<></>) : (
    <CurrentLibrary.Provider value={{
      id: libraryId,
      setContext,
      resetContext,
    }}>
      <rspc.Provider client={client2} queryClient={queryClient}>
        {libraryId ? (
          <>{children}</>
        ) : (
          <LibrariesSelect />
        )}
      </rspc.Provider>
    </CurrentLibrary.Provider>
  );
}
