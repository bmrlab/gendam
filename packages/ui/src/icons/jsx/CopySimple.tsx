import * as React from 'react'
import type { SVGProps } from 'react'
const SvgCopySimple = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <path
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.011}
      d="M13.562 11.54V2.438H4.46"
    />
    <path
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={1.011}
      d="M11.54 4.46H2.438v9.102h9.102z"
    />
  </svg>
)
export default SvgCopySimple
