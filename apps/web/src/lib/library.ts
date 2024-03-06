import { createContext, useContext } from "react";
import { Store } from "tauri-plugin-store-api";

const CURRENT_LIBRARY_KEY = "current-library-id";

export const CurrentLibraryStorage = {
  set: async (libraryId: string) => {
    if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
      const store = new Store("settings.json");
      await store.set(CURRENT_LIBRARY_KEY, libraryId);
      await store.save();
    } else if (typeof window !== 'undefined' && typeof window.localStorage !== 'undefined') {
      window.localStorage.setItem(CURRENT_LIBRARY_KEY, libraryId);
    } else {
      return null;
    }
  },

  get: async (): Promise<string|null> => {
    if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
      const store = new Store("settings.json");
      return await store.get(CURRENT_LIBRARY_KEY) ?? null;
    } else if (typeof window !== 'undefined' && typeof window.localStorage !== 'undefined') {
      return window.localStorage.getItem(CURRENT_LIBRARY_KEY) ?? null;
    } else {
      return null;
    }
  },

  reset: async () => {
    if (typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined') {
      const store = new Store("settings.json");
      await store.delete(CURRENT_LIBRARY_KEY);
      await store.save();
    } else if (typeof window !== 'undefined' && typeof window.localStorage !== 'undefined') {
      window.localStorage.removeItem(CURRENT_LIBRARY_KEY);
    } else {
      return null;
    }
  }
}

type CurrentLibraryContext = {
  id: string | null;
  setContext: (id: string) => Promise<void>;
  resetContext: () => Promise<void>;
};

export const CurrentLibrary = createContext<CurrentLibraryContext>({
  id: null,
  setContext: async () => {},
  resetContext: async () => {},
});
