"use client";
// import { useRouter } from 'next/navigation'
import { usePathname } from 'next/navigation'
import { useCallback, useEffect, useState, useMemo } from "react";
import { client, createClientWithLibraryId, queryClient, rspc } from "@/lib/rspc";
import { CurrentLibrary } from "@/lib/library";
import LibrariesSelect from "@/components/LibrariesSelect";

export default function ClientLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  // const router = useRouter();
  // const pathname = usePathname();

  const [pending, setPending] = useState(true);
  const [currentLibraryId, setCurrentLibraryId] = useState<string|null>(null);

  useEffect(() => {
    const libraryIdInStorage = window.localStorage.getItem("current-library-id") ?? null;
    // if (!libraryIdInStorage && pathname !== '/') {
    //   window.location.href = '/';
    //   // router.push('/library/new')
    // }
    setCurrentLibraryId(libraryIdInStorage);
    setPending(false);
  }, [setCurrentLibraryId]);

  const setCurrentLibrary = useCallback((id: string) => {
    setCurrentLibraryId(id);
    window.localStorage.setItem("current-library-id", id);
    setPending(true);
    /**
     * TODO: router.refresh 看起来会缓存 fetch 请求参数, 导致 rspc.useQuery 里 libraryId 没切换
     * 除非是切换到其他 tab 再切换回来才会更新, 这样就有问题, 展示改成刷新整个页面
     */
    // router.refresh();
    location.reload();
  }, [setCurrentLibraryId]);

  const resetCurrentLibrary = useCallback(() => {
    setCurrentLibraryId(null);
    window.localStorage.removeItem("current-library-id");
    // location.reload();
  }, [setCurrentLibraryId]);

  let client2 = useMemo(() => {
    return currentLibraryId ?
      createClientWithLibraryId(currentLibraryId) :
      client;
  }, [currentLibraryId]);

  return pending ? (<></>) : (
    <CurrentLibrary.Provider value={{
      id: currentLibraryId,
      setCurrentLibrary,
      resetCurrentLibrary,
    }}>
      <rspc.Provider client={client2} queryClient={queryClient}>
        {currentLibraryId ? (
          <>{children}</>
        ) : (
          <div className="bg-white w-screen h-screen flex items-center justify-center">
            <LibrariesSelect />
          </div>
        )}
      </rspc.Provider>
    </CurrentLibrary.Provider>
  );
}
