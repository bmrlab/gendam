import * as React from "react";
import type { SVGProps } from "react";
const SvgEditSquare = (props: SVGProps<SVGSVGElement>) => (
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
      d="m10.502 3.372 1.035-1.035a1.15 1.15 0 1 1 1.626 1.626L7.61 9.518a2.76 2.76 0 0 1-1.164.693l-1.647.49.491-1.646c.13-.44.369-.84.693-1.164zm0 0L12.12 4.99m.92 4.217v2.913a1.38 1.38 0 0 1-1.38 1.38H3.38A1.38 1.38 0 0 1 2 12.12V3.84a1.38 1.38 0 0 1 1.38-1.38h2.913"
    />
  </svg>
);
export default SvgEditSquare;
