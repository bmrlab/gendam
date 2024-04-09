import * as React from "react";
import type { SVGProps } from "react";
const SvgArrowDown = (props: SVGProps<SVGSVGElement>) => (
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
      d="M3 5.793c0 .186.068.339.192.468l4.284 4.386c.158.152.321.23.524.23a.68.68 0 0 0 .519-.23l4.29-4.386A.65.65 0 0 0 13 5.793a.66.66 0 0 0-.665-.67.7.7 0 0 0-.485.202L7.994 9.277 4.15 5.325a.7.7 0 0 0-.485-.202.66.66 0 0 0-.665.67"
      style={{
        mixBlendMode: "luminosity",
      }}
    />
  </svg>
);
export default SvgArrowDown;
