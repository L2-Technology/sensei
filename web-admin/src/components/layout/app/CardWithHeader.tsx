import React from "react";

const CardWithHeader = ({ header, children }) => {
  return (
    <div className="bg-white overflow-hidden shadow rounded-lg">
      <div className="border-b border-gray-200 px-4 py-5 sm:px-6">{header}</div>
      <div className="px-4 py-5 sm:p-6">{children}</div>
    </div>
  );
};

export default CardWithHeader;
