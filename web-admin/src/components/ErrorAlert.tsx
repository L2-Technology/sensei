import React, { useState, useEffect, PropsWithChildren } from "react";
import { XIcon, ExclamationIcon } from "@heroicons/react/outline";

interface AlertMsgProps {
  type: "error" | "warning";
  className?: string;
}

export const AlertMsg = ({
  children,
  type,
  className,
}: PropsWithChildren<AlertMsgProps>) => {
  const [mount, setMount] = useState(false);

  const Icon = type === "error" ? XIcon : ExclamationIcon;

  useEffect(() => {
    setMount(true);
  }, []);

  return (
    <>
      <div
        className={`${
          mount ? "opacity-100 " : "h-0 scale-y-0 opacity-0"
        } ${className} relative overflow-hidden rounded-xl bg-plum-200 p-2.5 shadow transition-all duration-500`}
      >
        <div className="flex items-center">
          <span className={`${type} w-8`}>
            <Icon className="h-6 w-6" />
          </span>

          <div className="flex-1 px-2">
            <p className="text-sm text-gray-text">{children}</p>
          </div>
        </div>
      </div>
    </>
  );
};

type ErrorAlertProps = {
  message: String;
  className: String;
};

const ErrorAlert = ({ message, className }: ErrorAlertProps) => {
  return (
    <div className={`rounded-md bg-red-50 p-4 ${className}`}>
      <div className="flex">
        <div className="flex-shrink-0">
          <svg
            className="h-5 w-5 text-red-400"
            fill="currentColor"
            viewBox="0 0 20 20"
          >
            <path
              fillRule="evenodd"
              d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
              clipRule="evenodd"
            />
          </svg>
        </div>
        <div className="ml-3">
          <p className="text-sm leading-5 font-medium text-red-800">
            {message}
          </p>
        </div>
      </div>
    </div>
  );
};

ErrorAlert.defaultProps = {
  className: "",
};

export default ErrorAlert;
