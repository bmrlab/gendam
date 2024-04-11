// const ListFilter = () => {
//   return (
//     <RadioGroup
//       defaultValue={filter as string}
//       className="flex px-8 py-4"
//       value={filter as string}
//       onValueChange={(filter) => setFilter(filter as TaskListRequestFilter)}
//     >
//       <div className="flex items-center space-x-2">
//         <RadioGroupItem value="all" id="task-filter-all" />
//         <Label htmlFor="task-filter-all">全部</Label>
//       </div>
//       <div className="flex items-center space-x-2">
//         <RadioGroupItem value="excludeCompleted" id="task-filter-excludeCompleted" />
//         <Label htmlFor="task-filter-excludeCompleted">未完成</Label>
//       </div>
//     </RadioGroup>
//   )
// }

// const bottomRef = useRef(null)
// useEffect(() => {
//   // 防止初次加载
//   const timer = setTimeout(() => {
//     const observer = new IntersectionObserver(
//       ([entry]) => {
//         if (entry.isIntersecting) {
//           if (hasNextPage) {
//             fetchNextPage()
//           }
//         }
//       },
//       {
//         root: null,
//         threshold: 0.1,
//       },
//     )
//     if (bottomRef.current) {
//       observer.observe(bottomRef.current)
//     }
//   }, 500)

//   return () => {
//     clearTimeout(timer)
//   }
// }, [fetchNextPage, hasNextPage])

// <div className="flex items-center justify-center p-4">
//   {hasNextPage ? (
//     <>
//       <div className="text-xs text-slate-400">滚动加载更多</div>
//       <div ref={bottomRef}></div>
//     </>
//   ) : (
//     <div className="text-xs text-slate-400">没有更多了</div>
//   )}
// </div>
