import { createContext, useContext } from "react";

type CurrentLibraryContext = {
  id: string | null;
  setCurrentLibrary: (id: string) => void;
};

export const CurrentLibrary = createContext<CurrentLibraryContext>({
  id: null,
  setCurrentLibrary: () => {},
});
