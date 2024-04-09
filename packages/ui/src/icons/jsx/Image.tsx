import * as React from "react";
import type { SVGProps } from "react";
const SvgImage = (props: SVGProps<SVGSVGElement>) => (
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
      d="M13.875 1.987H2.125C1.506 1.987 1 2.494 1 3.112v9.774c0 .619.506 1.125 1.125 1.125h11.75c.619 0 1.125-.506 1.125-1.125V3.112c0-.618-.506-1.125-1.125-1.125m0 10.896-.002.001H2.129l-.001-.002V9.558l2.9-2.965 3.395 4.217a.56.56 0 0 0 .737.094l2.508-1.696 2.208 1.04zM12.03 8.07l1.845.761-.002-5.717-.001-.002H2.127l-.002.002v4.834l2.519-2.576a.564.564 0 0 1 .825.021l3.47 4.3 2.434-1.642a.56.56 0 0 1 .657.019m-1.124-3.805a1.58 1.58 0 0 0-1.578 1.579c0 .87.708 1.578 1.578 1.578a1.58 1.58 0 0 0 1.578-1.578 1.58 1.58 0 0 0-1.578-1.579m-.453 1.579a.453.453 0 1 0 .907-.001.453.453 0 0 0-.907 0"
      clipRule="evenodd"
    />
  </svg>
);
export default SvgImage;
