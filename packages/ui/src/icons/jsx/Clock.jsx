import * as React from "react";
const SvgClock = (props) => (
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
      strokeMiterlimit={10}
      d="M8 14A6 6 0 1 0 8 2a6 6 0 0 0 0 12Z"
    />
    <path
      stroke="#000"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M8 4.5V8h3.5"
    />
  </svg>
);
export default SvgClock;
