import ClickableDiv from "src/components/layout/app/ClickableDiv";
import React, { useEffect, useState } from "react";

const SuccessAlert = ({ message, className }) => {
  const [open, setIsOpen] = useState(true);

  useEffect(() => {
    setIsOpen(true);
  }, [message]);

  if (!open) {
    return null;
  }

  return (
    <div className={`rounded-md bg-green-50 p-4 ${className}`}>
      <div className="flex">
        <div className="flex-shrink-0">
          <svg
            className="h-5 w-5 text-green-400"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fillRule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z"
              clipRule="evenodd"
            />
          </svg>
        </div>
        <div className="ml-3">
          <p className="text-sm leading-5 font-medium text-green-800">
            {message}
          </p>
        </div>
        <div className="ml-auto pl-3">
          <div className="-mx-1.5 -my-1.5">
            <ClickableDiv
              clickHandler={(event) => {
                event.preventDefault();
                event.stopPropagation();
                setIsOpen(false);
              }}
              className="inline-flex rounded-md p-1.5 text-green-500 hover:bg-green-100 focus:outline-none focus:bg-green-100 transition ease-in-out duration-150 cursor-pointer"
              aria-label="Dismiss"
            >
              <svg className="h-5 w-5" viewBox="0 0 20 20" fill="currentColor">
                <path
                  fillRule="evenodd"
                  d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
                  clipRule="evenodd"
                />
              </svg>
            </ClickableDiv>
          </div>
        </div>
      </div>
    </div>
  );
};

SuccessAlert.defaultProps = {
  className: "",
};

export default SuccessAlert;
