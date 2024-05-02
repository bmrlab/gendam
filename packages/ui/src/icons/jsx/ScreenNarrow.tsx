import * as React from 'react'
import type { SVGProps } from 'react'
const SvgScreenNarrow = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <path
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M13 6h-3V3M3 10h3v3M10 13v-3h3M6 3v3H3"
    />
  </svg>
)
export default SvgScreenNarrow
