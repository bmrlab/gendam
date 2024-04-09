import * as React from "react";
import type { SVGProps } from "react";
const SvgLoading = (props: SVGProps<SVGSVGElement>) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="16px"
    height="16px"
    fill="none"
    viewBox="0 0 16 16"
    {...props}
  >
    <g clipPath="url(#Loading_svg__a)">
      <path
        fill="currentColor"
        d="M15 8.56a.56.56 0 0 1-.56-.56c0-.93-.18-1.83-.54-2.68-.35-.81-.85-1.56-1.48-2.18a6.9 6.9 0 0 0-2.18-1.48c-.85-.35-1.75-.54-2.68-.54a.56.56 0 1 1 0-1.12c1.08 0 2.13.21 3.11.63.95.4 1.81.98 2.54 1.71s1.31 1.59 1.71 2.54c.43.99.64 2.04.64 3.12 0 .31-.25.56-.56.56"
      />
    </g>
    <defs>
      <clipPath id="Loading_svg__a">
        <path fill="#fff" d="M7 0h8.56v8.56H7z" />
      </clipPath>
    </defs>
  </svg>
);
export default SvgLoading;
