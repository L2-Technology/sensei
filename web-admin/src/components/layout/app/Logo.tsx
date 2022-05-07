import React from "react";

const Logo = () => {
  return (
    <div className="flex items-center flex-shrink-0 px-4">
      <img className="h-6 w-auto" src="/logo.png" alt="ServerlessHQ" />
    </div>
  );
};

Logo.defaultProps = {
  color: "text-white",
};

export default Logo;
