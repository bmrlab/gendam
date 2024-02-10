"use client";
import * as client from "@rspc/client";
import * as event from "@tauri-apps/api/event";
import * as window from "@tauri-apps/api/window";
import { Link } from '@rspc/client';

function newWsManager() {
  const activeMap = new Map();
  const listener = event.listen("plugin:rspc:transport:resp", (event) => {
    const results = JSON.parse(event.payload as string);
    for (const result of results) {
      const item = activeMap.get(result.id);
      if (!item) {
        console.error(
          `rspc: received event with id '${result.id}' for unknown`
        );
        return;
      }
      client._internal_fireResponse(result, {
        resolve: item.resolve,
        reject: item.reject
      });
      if (item.oneshot && result.type === "value" || result.type === "complete") {
        activeMap.delete(result.id);
      }
    }
  });
  return [
    activeMap,
    (data: any) => listener.then(
      () => window.appWindow.emit("plugin:rspc:transport", JSON.stringify(data))
    )
  ];
}

function tauriLink(): Link {
  const [activeMap, sendRequest] = newWsManager();
  return client._internal_wsLinkInternal([activeMap as any, sendRequest as any]);
}

export { tauriLink };
