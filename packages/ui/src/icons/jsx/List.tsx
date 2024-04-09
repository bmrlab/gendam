import * as React from "react";
import type { SVGProps } from "react";
const SvgList = (props: SVGProps<SVGSVGElement>) => (
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
      d="M1.1 4.338c.61 0 1.098-.487 1.098-1.099A1.098 1.098 0 1 0 1.1 4.34m3.637-.356H15.25A.74.74 0 0 0 16 3.24a.735.735 0 0 0-.75-.743H4.736a.74.74 0 0 0-.743.743c0 .418.325.743.743.743M1.1 9.1c.612 0 1.1-.488 1.1-1.1a1.098 1.098 0 1 0-1.1 1.1m3.638-.356H15.25A.74.74 0 0 0 16 8a.735.735 0 0 0-.75-.743H4.736A.74.74 0 0 0 3.994 8c0 .418.325.743.743.743M1.1 13.86c.612 0 1.1-.488 1.1-1.1a1.098 1.098 0 1 0-1.1 1.1m3.638-.356H15.25A.74.74 0 0 0 16 12.76a.735.735 0 0 0-.75-.743H4.736a.74.74 0 0 0-.743.743c0 .418.325.744.743.744"
      style={{
        mixBlendMode: "luminosity",
      }}
    />
  </svg>
);
export default SvgList;
