import * as React from 'react'
import type { SVGProps } from 'react'
const SvgSidebar = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <rect width={15} height={11} x={0.5} y={2.5} stroke="currentColor" rx={0.5} />
    <path stroke="currentColor" d="M9.5 3v10M11 4.5h3M11 6.5h3M11 8.5h3" />
  </svg>
)
export default SvgSidebar
