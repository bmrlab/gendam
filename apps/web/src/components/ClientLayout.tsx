"use client";
import { useCallback, useEffect, useState, useMemo, use } from "react";
import { client, queryClient, rspc } from "@/lib/rspc";
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { CurrentLibrary, type Library } from "@/lib/library";
import LibrariesSelect from "@/components/LibrariesSelect";

export default function ClientLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  const [pending, setPending] = useState(true);
  const [library, setLibrary] = useState<Library|null>(null);
  // const [homeDir, setHomeDir] = useState<string|null>(null);

  const blockCmdQ = useCallback(() => {
    document.addEventListener('keydown', event => {
      if (event.metaKey && (event.key === 'q' || event.key === 'w')) {
        event.preventDefault()
        console.log('Cmd + Q is pressed.')
        alert('Cmd + Q 暂时禁用了，因为会导致 qdrant 不正常停止，TODO：实现 Cmd + Q 按了以后自行处理 app 退出')
        /**
         * https://github.com/bmrlab/tauri-dam-test-playground/issues/21#issuecomment-2002549684
         */
      }
    })
  }, [])

  useEffect(() => {
    blockCmdQ()

    const p1 = client.query(["libraries.get_current_library"]).then(
      (library: Library) => setLibrary(library)
    ).catch(error => {
      console.log('libraries.get_current_library error:', error);
      setLibrary(null);
    });
    // const p2 = client.query(["files.home_dir"]).then((homeDir) => {
    //   setHomeDir(homeDir);
    // }).catch(error => {
    //   console.log('files.home_dir error:', error);
    //   setHomeDir(null);
    // });
    // Promise.all([p1, p2]).then(() => setPending(false));
    p1.then(() => setPending(false));
  }, [setLibrary, setPending, blockCmdQ]);

  const setContext = useCallback(async (library: Library) => {
    setLibrary(library);
    setPending(true);
    try {
      await client.mutation(["libraries.set_current_library", library.id]);
      // setPending(false);
      // 最后 reload 一下，用新的 library 请求数据过程中，页面上还残留着上个 library 已请求的数据
      // 既然要 reload，就不设置 setPending(false) 了
      location.reload();
    } catch(err) {
      console.error('CurrentLibraryStorage.set() error:', err);
    }
  }, [setLibrary]);

  const getFileSrc = useCallback((assetObjectHash: string) => {
    if (!library) {
      return '';
    }
    const fileFullPath = library.dir + '/files/' + assetObjectHash;
    if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
      return convertFileSrc(fileFullPath);
    } else {
      return `http://localhost:3001/file/localhost/${fileFullPath}`
    }
  }, [library]);

  return pending ? (<></>) : (
    <CurrentLibrary.Provider value={{
      ...(library ? library : {}),
      setContext,
      getFileSrc,
    }}>
      <rspc.Provider client={client} queryClient={queryClient}>
        {library?.id ? (
          <>{children}</>
        ) : (
          <LibrariesSelect />
        )}
      </rspc.Provider>
    </CurrentLibrary.Provider>
  );
}
