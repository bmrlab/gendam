import * as React from "react";
import type { SVGProps } from "react";
const SvgTrash = (props: SVGProps<SVGSVGElement>) => (
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
      stroke="currentColor"
      strokeWidth={0.293}
      d="M10.865 14.154H5.135a1.795 1.795 0 0 1-1.793-1.793V3.915h.649v8.446c0 .63.513 1.144 1.144 1.144h5.728c.632 0 1.145-.513 1.145-1.144V3.915h.649v8.446c0 .989-.805 1.793-1.793 1.793Z"
    />
    <path
      fill="currentColor"
      stroke="currentColor"
      strokeWidth={0.293}
      d="M13.839 4.24H2.16a.324.324 0 1 1 0-.65H13.84a.324.324 0 1 1 0 .65Z"
    />
    <path
      fill="currentColor"
      stroke="currentColor"
      strokeWidth={0.293}
      d="M10.206 3.915h-.649v-1.01a.41.41 0 0 0-.41-.41H6.852a.41.41 0 0 0-.41.41v1.01h-.649v-1.01a1.06 1.06 0 0 1 1.06-1.059h2.294a1.06 1.06 0 0 1 1.059 1.059zM6.28 11.667a.324.324 0 0 1-.324-.325v-4.76a.324.324 0 1 1 .648 0v4.76c0 .18-.145.325-.324.325ZM9.72 11.667a.324.324 0 0 1-.325-.325v-4.76a.324.324 0 1 1 .648 0v4.76c0 .18-.145.325-.324.325Z"
    />
  </svg>
);
export default SvgTrash;
