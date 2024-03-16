"use client";
import { useCallback, useEffect, useState, useMemo } from "react";
import { client, queryClient, rspc } from "@/lib/rspc";
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { CurrentLibrary } from "@/lib/library";
import LibrariesSelect from "@/components/LibrariesSelect";

export default function ClientLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const [pending, setPending] = useState(true);
  const [libraryId, setLibraryId] = useState<string|null>(null);
  const [homeDir, setHomeDir] = useState<string|null>(null);

  useEffect(() => {
    const p1 = client.query(["libraries.get_current_library"]).then((libraryIdInStorage) => {
      setLibraryId(libraryIdInStorage);
    }).catch(error => {
      console.log('libraries.get_current_library error:', error);
      setLibraryId(null);
    });
    const p2 = client.query(["files.home_dir"]).then((homeDir) => {
      setHomeDir(homeDir);
    }).catch(error => {
      console.log('files.home_dir error:', error);
      setHomeDir(null);
    });
    Promise.all([p1, p2]).then(() => setPending(false));
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

  const getFileSrc = useCallback((assetObjectHash: string) => {
    const fileFullPath = homeDir + '/' + assetObjectHash;
    if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
      return convertFileSrc(fileFullPath);
    } else {
      return `http://localhost:3001/file/localhost/${fileFullPath}`
    }
  }, [homeDir]);

  return pending ? (<></>) : (
    <CurrentLibrary.Provider value={{
      id: libraryId,
      setContext,
      getFileSrc,
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
