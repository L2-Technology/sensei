import React from "react";

const InfoAlert = ({ message, className }) => {
  return (
    <div className={`rounded-md bg-blue-50 p-4 ${className}`}>
      <div className="flex">
        <div className="flex-shrink-0">
          <svg width={24} className="text-blue-400" height={24} fill="none">
            <path
              d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
              stroke="currentColor"
              strokeWidth={2}
              strokeLinecap="round"
              strokeLinejoin="round"
            />
          </svg>
        </div>
        <div className="ml-3">
          <p className="text-sm leading-5 font-medium text-blue-800">
            {message}
          </p>
        </div>
      </div>
    </div>
  );
};

InfoAlert.defaultProps = {
  className: "",
};

export default InfoAlert;
