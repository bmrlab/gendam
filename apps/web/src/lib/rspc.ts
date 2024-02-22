"use client";
import { httpLink, initRspc } from "@rspc/client";
// import { tauriLink } from "@rspc/tauri";
import { tauriLink } from "./rspc-tauri";
import { createReactQueryHooks, QueryClient } from "@rspc/react";
import type { Procedures } from "@/lib/bindings";

export const createClientWithLibraryId = (libraryId: string) => {
  return initRspc<Procedures>({
    links: [
      typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined' ?
        tauriLink():
        httpLink({
          url: "http://localhost:3001/rspc",
          headers() {
            return {
              "X-Library-ID": libraryId,
            };
          },
        })
    ]
  });
}

export const client = initRspc<Procedures>({
  links: [
    typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined' ?
      tauriLink():
      httpLink({
        url: "http://localhost:3001/rspc",
      })
  ]
});

export const rspc = createReactQueryHooks<Procedures>(client);

export const queryClient: QueryClient = new QueryClient({
	defaultOptions: {
		queries: {
			suspense: false,
		},
		mutations: {
			onSuccess: () => queryClient.invalidateQueries()
		}
	}
});
