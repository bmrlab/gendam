import * as React from "react";
const SvgDownload = (props) => (
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
      d="M5.375 6.875 8 9.5l2.625-2.625M8 2.498v7"
    />
    <path
      stroke="#000"
      strokeLinecap="round"
      strokeLinejoin="round"
      d="M14 8.5V13a.5.5 0 0 1-.5.5h-11A.5.5 0 0 1 2 13V8.5"
    />
  </svg>
);
export default SvgDownload;
