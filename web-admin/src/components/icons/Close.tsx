import * as React from "react";

function SvgClose(props) {
  return (
    <svg width={24} height={24} stroke="currentColor" fill="none" {...props}>
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="2"
        d="M6 18L18 6M6 6l12 12"
      />
    </svg>
  );
}

export default SvgClose;
