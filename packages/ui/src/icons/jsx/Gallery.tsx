import * as React from 'react'
import type { SVGProps } from 'react'
const SvgGallery = (props: SVGProps<SVGSVGElement>) => (
  <svg xmlns="http://www.w3.org/2000/svg" width="16px" height="16px" fill="none" viewBox="0 0 16 16" {...props}>
    <rect width={15} height={7.62} x={0.5} y={2.24} stroke="currentColor" rx={1.5} />
    <rect width={2.4} height={2.4} y={11.86} fill="currentColor" rx={0.5} />
    <rect width={2.4} height={2.4} x={3.4} y={11.86} fill="currentColor" rx={0.5} />
    <rect width={2.4} height={2.4} x={6.8} y={11.86} fill="currentColor" rx={0.5} />
    <rect width={2.4} height={2.4} x={10.2} y={11.86} fill="currentColor" rx={0.5} />
    <rect width={2.4} height={2.4} x={13.6} y={11.86} fill="currentColor" rx={0.5} />
  </svg>
)
export default SvgGallery
