import { createContext } from "react";

type CurrentLibraryContext = {
  id: string | null;
  setContext: (id: string) => Promise<void>;
};

export const CurrentLibrary = createContext<CurrentLibraryContext>({
  id: null,
  setContext: async () => {},
});
