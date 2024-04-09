import * as React from "react";
const SvgRemoveSquare = (props) => (
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
      d="M13.04 9.207v2.913a1.38 1.38 0 0 1-1.38 1.38H3.38A1.38 1.38 0 0 1 2 12.12V3.84a1.38 1.38 0 0 1 1.38-1.38h2.913"
    />
    <path
      fill="#000"
      fillRule="evenodd"
      stroke="#000"
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={0.5}
      d="M8.573 1.573a.25.25 0 0 1 .354 0L11 3.646l2.073-2.073a.25.25 0 0 1 .354.354L11.354 4l2.073 2.073a.25.25 0 0 1-.354.354L11 4.354 8.927 6.427a.25.25 0 0 1-.354-.354L10.646 4 8.573 1.927a.25.25 0 0 1 0-.354"
      clipRule="evenodd"
    />
  </svg>
);
export default SvgRemoveSquare;
