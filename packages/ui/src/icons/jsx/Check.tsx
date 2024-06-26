import * as React from 'react'
import type { SVGProps } from 'react'
const SvgCheck = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <path stroke="currentColor" strokeLinecap="round" strokeLinejoin="round" d="m13.5 4.5-7 7L3 8" />
  </svg>
)
export default SvgCheck
