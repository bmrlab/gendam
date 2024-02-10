"use client";
import { httpLink, initRspc } from "@rspc/client";
import { createReactQueryHooks, QueryClient } from "@rspc/react";
// import { createReactQueryHooks, QueryClient } from "@tanstack/react-query";
import type { Procedures } from "@/lib/bindings";

// const getClient = async () => {
//   const links = [];
//   if (typeof window.__TAURI__ !== 'undefined') {
//     const { tauriLink } = await import("@rspc/tauri");
//     links.push(tauriLink());
//   } else {
//     links.push(httpLink({
//       url: "http://localhost:3001/rspc",
//     }));
//   }
//   const client = initRspc<Procedures>({ links });
//   return client;
// }

export const client = initRspc<Procedures>({
  links: [
    httpLink({
      url: "http://localhost:3001/rspc",
    })
  ]
});

export const rspc = createReactQueryHooks<Procedures>(client);

export const queryClient: QueryClient = new QueryClient({
	defaultOptions: {
		queries: {
			suspense: true
		},
		mutations: {
			onSuccess: () => queryClient.invalidateQueries()
		}
	}
});
