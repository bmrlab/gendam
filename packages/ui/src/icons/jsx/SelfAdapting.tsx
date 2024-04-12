import * as React from 'react'
import type { SVGProps } from 'react'
const SvgSelfAdapting = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <rect width={4} height={7} x={2} y={2} stroke="currentColor" rx={0.5} />
    <rect width={6} height={3} x={2} y={11} stroke="currentColor" rx={0.5} />
    <rect width={4} height={3} x={10} y={11} stroke="currentColor" rx={0.5} />
    <rect width={6} height={7} x={8} y={2} stroke="currentColor" rx={0.5} />
  </svg>
)
export default SvgSelfAdapting
