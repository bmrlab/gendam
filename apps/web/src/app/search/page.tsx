"use client";
import Image from "next/image";
import { useCallback, useEffect, useState, useRef } from "react";
import { rspc, client } from "@/lib/rspc";
import type { SearchResultPayload } from "@/lib/bindings";

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
      <div>
        {data?.map((item: SearchResultPayload) => {
          // return item.fullPath;
          return (
            <Image src={"http://localhost:3001/assets/" + item.fullPath} alt={item.fullPath} key={item.fullPath} width={300} height={300} />
          )
        })}
      </div>
    </main>
  );
}
