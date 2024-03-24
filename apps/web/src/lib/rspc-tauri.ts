import { OperationType, RSPCError, Transport, randomId } from '@rspc/client'
import { UnlistenFn, listen } from '@tauri-apps/api/event'
// import { appWindow } from '@tauri-apps/api/window'

export class TauriTransport implements Transport {
  private requestMap = new Map<string, (data: any) => void>()
  private listener?: Promise<UnlistenFn>
  clientSubscriptionCallback?: (id: string, value: any) => void

  constructor() {
    this.listener = listen('plugin:rspc:transport:resp', (event) => {
      const { id, result } = event.payload as any
      if (result.type === 'event') {
        if (this.clientSubscriptionCallback) this.clientSubscriptionCallback(id, result.data)
      } else if (result.type === 'response') {
        if (this.requestMap.has(id)) {
          this.requestMap.get(id)?.({ type: 'response', result: result.data })
          this.requestMap.delete(id)
        }
      } else if (result.type === 'error') {
        const { message, code } = result.data
        if (this.requestMap.has(id)) {
          this.requestMap.get(id)?.({ type: 'error', message, code })
          this.requestMap.delete(id)
        }
      } else {
        console.error(`Received event of unknown method '${result.type}'`)
      }
    })
  }

  async doRequest(operation: OperationType, key: string, input: any): Promise<any> {
    if (!this.listener) {
      await this.listener
    }

    const id = randomId()
    let resolve: (data: any) => void
    const promise = new Promise((res) => {
      resolve = res
    })

    // @ts-ignore
    this.requestMap.set(id, resolve)

    const { appWindow } = await import('@tauri-apps/api/window')
    await appWindow.emit('plugin:rspc:transport', {
      id,
      method: operation,
      params: {
        path: key,
        input,
      },
    })

    const body = (await promise) as any
    if (body.type === 'error') {
      const { code, message } = body
      throw new RSPCError(code, message)
    } else if (body.type === 'response') {
      return body.result
    } else {
      throw new Error(`RSPC Tauri doRequest received invalid body type '${body?.type}'`)
    }
  }
}

// import * as client from "@rspc/client";
// import * as event from "@tauri-apps/api/event";
// // import * as window from "@tauri-apps/api/window";
// import { Link } from '@rspc/client';

// function newWsManager() {
//   const activeMap = new Map();
//   const listener = event.listen("plugin:rspc:transport:resp", (event) => {
//     const results = JSON.parse(event.payload as string);
//     for (const result of results) {
//       const item = activeMap.get(result.id);
//       if (!item) {
//         console.error(
//           `rspc: received event with id '${result.id}' for unknown`
//         );
//         return;
//       }
//       client._internal_fireResponse(result, {
//         resolve: item.resolve,
//         reject: item.reject
//       });
//       if (item.oneshot && result.type === "value" || result.type === "complete") {
//         activeMap.delete(result.id);
//       }
//     }
//   });
//   return [
//     activeMap,
//     (data: any) => listener.then(
//       () => {
//         import("@tauri-apps/api/window").then((window) => {
//           window.appWindow.emit("plugin:rspc:transport", JSON.stringify(data))
//         });
//       }
//     )
//   ];
// }

// function tauriLink(): Link {
//   const [activeMap, sendRequest] = newWsManager();
//   return client._internal_wsLinkInternal([activeMap as any, sendRequest as any]);
// }

// export { tauriLink };
