import * as React from 'react'
import type { SVGProps } from 'react'
const SvgGrid = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <rect width={5} height={5} x={2} y={2} stroke="currentColor" rx={0.5} />
    <rect width={5} height={5} x={9} y={2} stroke="currentColor" rx={0.5} />
    <rect width={5} height={5} x={2} y={9} stroke="currentColor" rx={0.5} />
    <rect width={5} height={5} x={9} y={9} stroke="currentColor" rx={0.5} />
  </svg>
)
export default SvgGrid
