import * as React from "react";
const SvgQuestionMark = (props) => (
  <svg
    xmlns="http://www.w3.org/2000/svg"
    width="16px"
    height="16px"
    fill="none"
    viewBox="0 0 16 16"
    {...props}
  >
    <path fill="#fff" fillOpacity={0.01} d="M0 0h16v16H0z" />
    <path
      fill="#000"
      fillRule="evenodd"
      d="M5.188 4.435c0-1.228 1.222-2.247 2.506-2.247s2.505 1.019 2.505 2.247c0 1.135-.576 1.668-1.35 2.355l-.032.028C8.05 7.498 7.1 8.341 7.1 10.015a.594.594 0 1 0 1.188 0c0-1.121.573-1.647 1.35-2.336l.021-.02c.772-.684 1.728-1.532 1.728-3.224C11.387 2.397 9.47 1 7.694 1S4 2.397 4 4.435a.594.594 0 1 0 1.188 0M7.694 14a.904.904 0 1 0 0-1.808.904.904 0 0 0 0 1.808"
      clipRule="evenodd"
    />
  </svg>
);
export default SvgQuestionMark;
