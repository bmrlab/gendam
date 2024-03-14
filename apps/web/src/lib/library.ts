import { createContext } from "react";

type CurrentLibraryContext = {
  id: string | null;
  setContext: (id: string) => Promise<void>;
  getFileSrc: (assetObjectId: number) => string;
};

export const CurrentLibrary = createContext<CurrentLibraryContext>({
  id: null,
  setContext: async () => {},
  getFileSrc: (assetObjectId: number) => `http://localhost/${assetObjectId}`,  // 无效的默认值
});
