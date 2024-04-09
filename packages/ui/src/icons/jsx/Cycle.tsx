import * as React from "react";
import type { SVGProps } from "react";
const SvgCycle = (props: SVGProps<SVGSVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="16px"
    height="16px"
    fill="none"
    viewBox="0 0 16 16"
    {...props}
  >
    <path
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M11.01 6.232h3v-3"
    />
    <path
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M4.11 4.11a5.5 5.5 0 0 1 7.779 0l2.121 2.122M4.99 9.768h-3v3"
    />
    <path
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M11.889 11.889a5.5 5.5 0 0 1-7.778 0L1.99 9.768"
    />
  </svg>
);
export default SvgCycle;
