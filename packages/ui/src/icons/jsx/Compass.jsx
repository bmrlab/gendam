import * as React from "react";
const SvgCompass = (props) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="16px"
    height="16px"
    fill="none"
    viewBox="0 0 16 16"
    {...props}
  >
    <path
      fill="#000"
      fillRule="evenodd"
      d="M8 1a7 7 0 0 0-7 7 7 7 0 0 0 7 7 7 7 0 0 0 7-7 7 7 0 0 0-7-7M6.763 6.763a1.74 1.74 0 0 1 1.364-.509l2.966-1.348-1.348 2.966c.035.489-.134.99-.509 1.364a1.74 1.74 0 0 1-1.364.509l-2.966 1.348 1.348-2.966c-.035-.49.134-.99.509-1.364m.696 1.779A.765.765 0 1 0 8.54 7.46.765.765 0 0 0 7.46 8.54M1.984 8a6.016 6.016 0 1 0 12.032 0A6.016 6.016 0 0 0 1.984 8"
      clipRule="evenodd"
    />
  </svg>
);
export default SvgCompass;
