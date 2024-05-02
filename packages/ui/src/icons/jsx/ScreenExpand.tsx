import * as React from 'react'
import type { SVGProps } from 'react'
const SvgScreenExpand = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <path
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M10.5 3H13v2.5M5.5 13H3v-2.5M13 10.5V13h-2.5M3 5.5V3h2.5"
    />
  </svg>
)
export default SvgScreenExpand
