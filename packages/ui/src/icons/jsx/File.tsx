import * as React from "react";
import type { SVGProps } from "react";
const SvgFile = (props: SVGProps<SVGSVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="16px"
    height="16px"
    fill="none"
    viewBox="0 0 16 16"
    {...props}
  >
    <path
      fill="currentColor"
      fillRule="evenodd"
      d="M3.5 1.5a1 1 0 0 0-1 1v11a1 1 0 0 0 1 1h9a1 1 0 0 0 1-1v-8a.5.5 0 0 0-.148-.356L9.854 1.646A.5.5 0 0 0 9.5 1.5zm5.5 1H3.5v11h9V6h-3a.5.5 0 0 1-.5-.5zM11.793 5H10V3.207z"
      clipRule="evenodd"
    />
  </svg>
);
export default SvgFile;
