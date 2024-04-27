// 'use client'
// import Icon from '@/components/Icon'
// import PageNav from '@/components/PageNav'
// import Viewport from '@/components/Viewport'
// import { cn, twx } from '@/lib/utils'
// import { Input } from '@gendam/ui/v1/input'
// import { HTMLAttributes } from 'react'
// import { useBoundStore } from '../_store'

// const _Input = twx(Input)`max-w-[320px] bg-black/5 border-[0.5px] border-black/5 caret-[#017AFF]
// placeholder:text-[#676C77] placeholder:font-normal placeholder:text-[13px] placeholder:leading-[16px]
// focus:outline-none focus-visible:ring-2 focus-visible:ring-[#017AFF]
// `

// export default function VideoTaskHeader({ className }: HTMLAttributes<HTMLDivElement>) {
//   const searchKey = useBoundStore.use.searchKey()
//   const setSearchKey = useBoundStore.use.setSearchKey()

//   return (
//     <Viewport.Toolbar className="items-center justify-between">
//       <PageNav title="任务列表" />
//       <div className="relative">
//         <div className="absolute left-2 top-1/2 size-[14px] translate-y-[-50%] text-[#676C77]">
//           <Icon.search />
//         </div>
//         <_Input
//           value={searchKey}
//           onChange={(e) => setSearchKey(e.target.value ?? '')}
//           placeholder="搜索"
//           className={cn('h-full px-2 py-[7px]', Icon && 'pl-[28px]', className)}
//         />
//       </div>
//       <div className="flex items-center gap-0.5 justify-self-end text-[#676C77]">
//         <IconButton>
//           <Icon.grid className="size-4 text-[#797979]" />
//         </IconButton>
//         <IconButton>
//           <Icon.list className="size-4 text-[#797979]" />
//         </IconButton>
//         <IconButton>
//           <Icon.column className="size-4 text-[#797979]" />
//         </IconButton>
//       </div>
//     </Viewport.Toolbar>
//   )
// }

// const IconButton = twx.div`h-6 w-[28px] cursor-pointer rounded px-1.5 py-1 hover:bg-[#EBECEE]`
