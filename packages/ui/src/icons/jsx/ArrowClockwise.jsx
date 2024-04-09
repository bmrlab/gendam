import * as React from "react";
const SvgArrowClockwise = (props) => (
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
      d="M11.01 6.232h3v-3"
    />
    <path
      stroke="#000"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M11.89 11.89a5.5 5.5 0 1 1 0-7.78l2.12 2.122"
    />
  </svg>
);
export default SvgArrowClockwise;
