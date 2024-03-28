'use client'
import { FetchTransport, createClient } from '@rspc/client'
import { createReactQueryHooks } from '@rspc/react'
// import { QueryClient } from '@tanstack/react-query'
import { TauriTransport } from './rspc-tauri'
// import { TauriTransport } from '@rspc/tauri'

import type { Procedures } from '@/lib/bindings'

export const client = createClient<Procedures>({
  transport:
    typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined'
      ? new TauriTransport()
      : new FetchTransport('http://localhost:3001/rspc'),
})

// export const queryClient: QueryClient = new QueryClient({
//   defaultOptions: {
//     queries: {
//       retry: false,
//       refetchOnWindowFocus: false,
//     },
//     mutations: {
//       onSuccess: () => queryClient.invalidateQueries(),
//       onError: (error) => console.error(error),
//     },
//   },
// })

export const rspc = createReactQueryHooks<Procedures>()

// export const {
//   useContext,
//   useMutation,
//   useQuery,
//   useSubscription,
//   Provider: RSPCProvider,
// } = createReactQueryHooks<Procedures>();
