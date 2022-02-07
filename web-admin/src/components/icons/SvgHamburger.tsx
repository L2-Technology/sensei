import * as React from "react";

const SvgHamburger = (props) => {
  return (
    <svg
      stroke="currentColor"
      width={24}
      height={24}
      fill="none"
      viewBox="0 0 24 24"
      {...props}
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="2"
        d="M4 6h16M4 12h16M4 18h7"
      />
    </svg>
  );
};

export default SvgHamburger;
