import React, { useContext, createContext, useState, useMemo } from "react";

const WarningIcon = () => {
  return (
    <svg
      className="h-6 w-6 text-red-600"
      stroke="currentColor"
      fill="none"
      viewBox="0 0 24 24"
    >
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth="2"
        d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
      />
    </svg>
  );
};

interface ConfirmContextOptions {
  isOpen: boolean;
  ctaText: string;
  Icon: React.FC;
  color: string;
  title: string;
  description: string;
  callback(data?: any): void;
  cancelCallback(): void;
  callbackData: any;
}

const defaultOptions: ConfirmContextOptions = {
  isOpen: false,
  ctaText: "OK",
  Icon: WarningIcon,
  color: "red",
  title: "Are you sure",
  description: "Are you sure you want to take this action?",
  callback: () => {},
  cancelCallback: () => {},
  callbackData: null,
};

const ConfirmContext = createContext(null);

function useConfirm() {
  const context = useContext(ConfirmContext);
  if (!context) {
    throw new Error(`useConfirm must be used within a ConfirmProvider`);
  }

  const [confirm, setConfirm]: [
    ConfirmContextOptions,
    React.Dispatch<React.SetStateAction<ConfirmContextOptions>>
  ] = context;

  const showConfirm = (options) => {
    setConfirm({
      ...defaultOptions,
      ...options,
      isOpen: true,
    });
  };

  const hideConfirm = () => {
    setConfirm({
      ...confirm,
      isOpen: false,
    });
  };

  return {
    ...confirm,
    setConfirm,
    showConfirm,
    hideConfirm,
  };
}

const ConfirmProvider = (props) => {
  const [confirm, setConfirm] = useState<ConfirmContextOptions>(defaultOptions);
  const value = useMemo(() => [confirm, setConfirm], [confirm]);
  return <ConfirmContext.Provider value={value} {...props} />;
};

export { ConfirmProvider, useConfirm };
