import React from "react";

const DownCaret = (props) => {
  return (
    <svg
      className="-mr-1 ml-2 h-5 w-5"
      stroke="currentColor"
      fill="currentColor"
      viewBox="0 0 20 20"
      {...props}
    >
      <path
        fillRule="evenodd"
        d="M5.293 7.293a1 1 0 011.414 0L10 10.586l3.293-3.293a1 1 0 111.414 1.414l-4 4a1 1 0 01-1.414 0l-4-4a1 1 0 010-1.414z"
        clipRule="evenodd"
      />
    </svg>
  );
};

export default DownCaret;
