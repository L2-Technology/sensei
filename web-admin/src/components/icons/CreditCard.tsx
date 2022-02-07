import * as React from "react";

function SvgCreditCard(props) {
  return (
    <svg stroke="currentColor" width={24} height={24} fill="none" {...props}>
      <path
        d="M3 10h18M7 15h1m4 0h1m-7 4h12a3 3 0 003-3V8a3 3 0 00-3-3H6a3 3 0 00-3 3v8a3 3 0 003 3z"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export default SvgCreditCard;
