import React from "react";
import sensei from "../utils/sensei";

interface NodeStatus {
  created: boolean;
  running: boolean;
  authenticated: boolean;
  username?: string;
  alias?: string;
  pubkey?: string;
  role?: number;
}

interface AuthContextType {
  status: NodeStatus;
  create: (
    username: string,
    alias: string,
    passphrase: string,
    start: boolean
  ) => Promise<void>;
  login: (username: string, passphrase: string) => Promise<void>;
  logout: () => Promise<void>;
  isAdmin: () => boolean;
}

let AuthContext = React.createContext<AuthContextType>(null!);

const AuthProvider = ({
  initialStatus,
  children,
}: {
  initialStatus: NodeStatus;
  children: React.ReactNode;
}) => {
  let [status, setStatus] = React.useState<NodeStatus>(initialStatus);

  let isAdmin = () => {
    return status.role === 0;
  };

  let create = async (
    username: string,
    alias: string,
    passphrase: string,
    start: boolean
  ) => {
    let response = await sensei.init({
      username,
      alias,
      passphrase,
      start
    });
    setStatus((status) => {
      return {
        ...status,
        created: true,
        running: start,
        authenticated: true,
        alias,
        username,
        pubkey: response.pubkey,
        role: response.role,
      };
    });
  };

  let login = async (username: string, passphrase: string) => {
    let response = await sensei.login(username, passphrase);
    setStatus((status) => {
      return {
        ...status,
        username,
        alias: response.alias,
        running: true,
        authenticated: true,
        pubkey: response.pubkey,
        role: response.role,
      };
    });
  };

  let logout = async () => {
    await sensei.logout();
    setStatus(null);
  };

  let value = { status, create, login, logout, isAdmin };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
};

function useAuth() {
  return React.useContext(AuthContext);
}

export { AuthProvider, useAuth };
