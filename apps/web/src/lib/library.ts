import { createContext, useContext } from "react";

type CurrentLibraryContext = {
  id: string | null;
  setCurrentLibrary: (id: string) => void;
  resetCurrentLibrary: () => void;
};

export const CurrentLibrary = createContext<CurrentLibraryContext>({
  id: null,
  setCurrentLibrary: () => {},
  resetCurrentLibrary: () => {},
});
