"use client";
import { useCallback, useEffect, useState, useRef } from "react";
import { rspc, client } from "@/lib/rspc";

export default function Search() {
  const { data, isLoading, error } = rspc.useQuery(["video.search.all", "car"]);
  console.log(data);
  // useEffect(() => {
  //   client.query(["video.search_videos", "car"]).then((res) => {
  //     console.log("!!!", res);
  //   });
  // }, [])

  return (
    <main className="min-h-screen p-12">
      { JSON.stringify(data) }
    </main>
  );
}
