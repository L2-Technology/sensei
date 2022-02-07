import React, {
  createContext,
  ReactNode,
  useContext,
  useMemo,
  useState,
} from "react";

interface ModalContextOptions {
  isOpen: boolean;
  component: ReactNode | null;
}

const defaultOptions: ModalContextOptions = {
  isOpen: false,
  component: null,
};

const ModalContext = createContext(null);

function useModal() {
  const context = useContext(ModalContext);
  if (!context) {
    throw new Error(`useModal must be used within a ModalProvider`);
  }

  const [modal, setModal]: [
    ModalContextOptions,
    React.Dispatch<React.SetStateAction<ModalContextOptions>>
  ] = context;

  const showModal = (options) => {
    setModal({
      ...defaultOptions,
      ...options,
      isOpen: true,
    });
  };

  const hideModal = () => {
    setModal({
      ...modal,
      isOpen: false,
    });
  };

  return {
    ...modal,
    setModal,
    showModal,
    hideModal,
  };
}

const ModalProvider = (props) => {
  const [modal, setModal] = useState<ModalContextOptions>(defaultOptions);
  const value = useMemo(() => [modal, setModal], [modal]);
  return <ModalContext.Provider value={value} {...props} />;
};

export { ModalProvider, useModal };
