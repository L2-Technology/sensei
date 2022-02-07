import React from "react";

const ClickableDiv = ({ className, clickHandler, children }) => {
  return (
    <div
      className={className}
      onClick={clickHandler}
      onKeyPress={clickHandler}
      role="button"
      tabIndex={0}
    >
      {children}
    </div>
  );
};

export default ClickableDiv;
