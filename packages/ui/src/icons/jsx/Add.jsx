import * as React from "react";
const SvgAdd = (props) => (
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
      d="M2.5 8h11M8 2.5v11"
    />
  </svg>
);
export default SvgAdd;
