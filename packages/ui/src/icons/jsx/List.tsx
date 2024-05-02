import * as React from 'react'
import type { SVGProps } from 'react'
const SvgList = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <circle cx={1} cy={4} r={1} fill="currentColor" />
    <rect width={12} height={1} x={4} y={3.5} fill="currentColor" rx={0.5} />
    <circle cx={1} cy={8} r={1} fill="currentColor" />
    <rect width={12} height={1} x={4} y={7.5} fill="currentColor" rx={0.5} />
    <circle cx={1} cy={12} r={1} fill="currentColor" />
    <rect width={12} height={1} x={4} y={11.5} fill="currentColor" rx={0.5} />
  </svg>
)
export default SvgList
