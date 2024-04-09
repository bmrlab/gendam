import * as React from "react";
const SvgArrowUpLeft = (props) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="16px"
    height="16px"
    fill="none"
    viewBox="0 0 16 16"
    {...props}
  >
    <path
      stroke="#000"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="m5 8.5-3-3 3-3"
    />
    <path
      stroke="#000"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M5 12.5h5.5A3.5 3.5 0 0 0 14 9v0a3.5 3.5 0 0 0-3.5-3.5H2"
    />
  </svg>
);
export default SvgArrowUpLeft;
