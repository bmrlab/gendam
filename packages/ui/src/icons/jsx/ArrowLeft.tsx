import * as React from "react";
import type { SVGProps } from "react";
const SvgArrowLeft = (props: SVGProps<SVGSVGElement>) => (
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
      d="M10.207 3a.65.65 0 0 0-.468.192L5.353 7.476c-.152.158-.23.321-.23.524 0 .197.073.372.23.519l4.386 4.29c.13.123.282.191.468.191a.66.66 0 0 0 .67-.665.7.7 0 0 0-.202-.485L6.723 7.994l3.952-3.844a.7.7 0 0 0 .203-.485.66.66 0 0 0-.671-.665"
      style={{
        mixBlendMode: "luminosity",
      }}
    />
  </svg>
);
export default SvgArrowLeft;
