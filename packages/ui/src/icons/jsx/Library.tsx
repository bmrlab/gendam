import * as React from 'react'
import type { SVGProps } from 'react'
const SvgLibrary = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <path
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M8 14V8.5m2.5 5.5V8.5m-5 5.5V8.5M2 6l6-4 6 4m-1 8V6.888a32.362 32.362 0 0 0-10 0V14m-1 0h12M8 4.5h.005v.005H8z"
    />
  </svg>
)
export default SvgLibrary
