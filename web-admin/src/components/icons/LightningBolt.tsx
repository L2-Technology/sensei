import * as React from "react";

function SvgLightningBolt(props) {
  return (
    <svg viewBox="0 0 24 24" width="24" height="24" fill="none" {...props}>
      <path
        d="M13 10V3L4 14h7v7l9-11h-7z"
        stroke="currentColor"
        strokeWidth={2}
        strokeLinecap="round"
        strokeLinejoin="round"
      />
    </svg>
  );
}

export default SvgLightningBolt;
