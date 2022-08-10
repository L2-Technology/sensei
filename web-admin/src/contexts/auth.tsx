import React from "react";
import sensei from "../utils/sensei";

interface SenseiStatus {
  version: string;
  setup: boolean;
  authenticatedAdmin: boolean;
  authenticatedNode: boolean;
  username?: string;
  alias?: string;
  pubkey?: string;
  role?: number;
}

interface AuthContextType {
  status: SenseiStatus;
  init: (
    username: string,
    passphrase: string,
  ) => Promise<void>;
  loginAdmin: (username: string, passphrase: string) => Promise<void>;
  loginNode: (username: string, passphrase: string) => Promise<void>;
  logout: () => Promise<void>;
}

let AuthContext = React.createContext<AuthContextType>(null!);

const AuthProvider = ({
  initialStatus,
  children,
}: {
  initialStatus: SenseiStatus;
  children: React.ReactNode;
}) => {
  let [status, setStatus] = React.useState<SenseiStatus>(initialStatus);

  let init = async (
    username: string,
    passphrase: string,
  ) => {
    let response = await sensei.init({
      username,
      passphrase,
    });
    setStatus((status) => {
      return {
        ...status,
        setup: true,
        authenticatedAdmin: true,
        authenticatedNode: false,
        username,
      };
    });
  };

  let loginAdmin = async (username: string, passphrase: string) => {
    let response = await sensei.loginAdmin(username, passphrase);
    setStatus((status) => {
      return {
        ...status,
        username,
        authenticatedAdmin: true,
        authenticatedNode: false
      };
    });
  };

  let loginNode = async (username: string, passphrase: string) => {
    let response = await sensei.loginNode(username, passphrase);
    setStatus((status) => {
      return {
        ...status,
        username,
        alias: response.alias,
        authenticatedAdmin: false,
        authenticatedNode: true,
        pubkey: response.pubkey
      };
    });
  };

  let logout = async () => {
    await sensei.logout();
    setStatus(null);
  };

  let value = { status, init, loginAdmin, loginNode, logout };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

function useAuth() {
  return React.useContext(AuthContext);
}

export { AuthProvider, useAuth };
