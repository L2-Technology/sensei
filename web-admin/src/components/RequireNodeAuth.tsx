import { useAuth } from "../contexts/auth";
import { useLocation, Navigate } from "react-router";

const RequireNodeAuth = ({ children }: { children: JSX.Element }) => {
  let auth = useAuth();
  let location = useLocation();

  if (!auth.status.setup) {
    return <Navigate to="/setup" state={{ from: location }} />;
  }

  if (!auth.status.authenticatedNode) {
    return <Navigate to="/login" state={{ from: location }} />;
  }

  return children;
};

export default RequireNodeAuth;
