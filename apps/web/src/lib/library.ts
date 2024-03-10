import { createContext, useContext } from "react";
import { client } from "./rspc";
// import { Store } from "tauri-plugin-store-api";

const CURRENT_LIBRARY_KEY = "current-library-id";

export const CurrentLibraryStorage = {
  set: async (libraryId: string) => {
    await client.mutation(["libraries.set_current_library", libraryId]);
  },
  get: async (): Promise<string|null> => {
    return await client.query(["libraries.get_current_library"]);
  },
}

type CurrentLibraryContext = {
  id: string | null;
  setContext: (id: string) => Promise<void>;
};

export const CurrentLibrary = createContext<CurrentLibraryContext>({
  id: null,
  setContext: async () => {},
});
