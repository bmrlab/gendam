import * as React from 'react'
import type { SVGProps } from 'react'
const SvgColumn = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <rect width={15} height={11.62} x={0.5} y={2.19} stroke="currentColor" rx={0.5} />
    <path stroke="currentColor" d="M5.5 2.478v11M10.5 2.478v11" />
  </svg>
)
export default SvgColumn
