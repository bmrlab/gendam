import * as React from "react";
import type { SVGProps } from "react";
const SvgColumn = (props: SVGProps<SVGSVGElement>) => (
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
      d="M2.221 14.309h11.551c1.465 0 2.228-.756 2.228-2.201V3.892c0-1.445-.763-2.201-2.228-2.201H2.222C.761 1.69 0 2.44 0 3.89v8.217c0 1.444.763 2.2 2.221 2.2m.088-1.344c-.621 0-.966-.324-.966-.979V4.013c0-.655.345-.979.966-.979H4.9v9.931zm3.909 0v-9.93h3.564v9.93zm7.466-9.93c.622 0 .966.323.966.978v7.973c0 .655-.344.98-.966.98h-2.592V3.033z"
      style={{
        mixBlendMode: "luminosity",
      }}
    />
  </svg>
);
export default SvgColumn;
