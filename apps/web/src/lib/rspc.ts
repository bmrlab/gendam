'use client'
import { FetchTransport, NoOpTransport, RSPCError, WebsocketTransport, createClient } from '@rspc/client'
import { createReactQueryHooks } from '@rspc/react'
import { QueryClient } from '@tanstack/react-query'
import { toast } from 'sonner'
import { TauriTransport } from './rspc-tauri'
// import { TauriTransport } from '@rspc/tauri'

import type { Procedures } from '@/lib/bindings'

export const client = createClient<Procedures>({
  transport:
    typeof window !== 'undefined' && typeof window.__TAURI__ !== 'undefined'
      ? new TauriTransport()
      : typeof window !== 'undefined'
        ? new WebsocketTransport('ws://localhost:3001/rspc/ws')
        // ? new FetchTransport('http://localhost:3001/rspc')
        : new NoOpTransport(),
})

/**
 * 这个配置只对 useQuery 和 useMutation 有效, 对使用 client.query 和 client.mutation 调用的请求无效
 */
export const queryClient: QueryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: false,
      refetchOnWindowFocus: false,
    },
    mutations: {
      onSuccess: (data, variables, context) => {
        /**
         * invalidateQueries 会让所有的 query 都重新请求, 这样在 mutation 结束以后让页面上的数据刷新
         */
        // queryClient.invalidateQueries({
        //   queryKey: ['assets.list'],
        //   // refetchType: 'none',
        // })
        // toast.success('请求成功')
      },
      onError: (error) => {
        console.error(error)
        if (error instanceof RSPCError) {
          toast.error(`Request Error (code: ${error.code})`, {
            description: error.message,
          })
        } else {
          toast.error(`Request Error`, {
            description: error.message,
          })
        }
      },
    },
  },
})

export const rspc = createReactQueryHooks<Procedures>()

// export const {
//   useContext,
//   useMutation,
//   useQuery,
//   useSubscription,
//   Provider: RSPCProvider,
// } = createReactQueryHooks<Procedures>();
