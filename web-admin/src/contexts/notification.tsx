import { InboxIcon } from "@heroicons/react/outline";
import React, {
  createContext,
  ReactNode,
  useContext,
  useMemo,
  useState,
} from "react";

interface NotificationContextOptions {
  isOpen: boolean;
  component: ReactNode | null;
  iconComponent: ReactNode | null;
}

const defaultOptions: NotificationContextOptions = {
  isOpen: false,
  component: null,
  iconComponent: null
};

const NotificationContext = createContext(null);

function useNotification() {
  const context = useContext(NotificationContext);
  if (!context) {
    throw new Error(
      `useNotification must be used within a NotificationProvider`
    );
  }

  const [notification, setNotification]: [
    NotificationContextOptions,
    React.Dispatch<React.SetStateAction<NotificationContextOptions>>
  ] = context;

  const showNotification = (options) => {
    setNotification({
      ...defaultOptions,
      ...options,
      isOpen: true,
    });
  };

  const hideNotification = () => {
    setNotification({
      ...defaultOptions,
      isOpen: false,
    });
  };

  return {
    ...notification,
    setNotification,
    showNotification,
    hideNotification,
  };
}

const NotificationProvider = (props) => {
  const [notification, setNotification] =
    useState<NotificationContextOptions>(defaultOptions);
  const value = useMemo(() => [notification, setNotification], [notification]);
  return <NotificationContext.Provider value={value} {...props} />;
};

export { NotificationProvider, useNotification };
