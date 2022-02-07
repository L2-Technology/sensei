import React, { useContext, createContext, useState, useMemo } from "react";

const ErrorContext = createContext(null);

function useError() {
  const context = useContext(ErrorContext);
  if (!context) {
    throw new Error(`useError must be used within a ErrorProvider`);
  }

  const [error, setError]: [
    string,
    React.Dispatch<React.SetStateAction<string | null>>
  ] = context;

  const showError = (message) => {
    setError(message);
  };

  const hideError = () => {
    setError(null);
  };

  return {
    error,
    setError,
    showError,
    hideError,
  };
}

const ErrorProvider = (props) => {
  const [error, setError] = useState(null);
  const value = useMemo(() => [error, setError], [error]);
  return <ErrorContext.Provider value={value} {...props} />;
};

export { ErrorProvider, useError };
