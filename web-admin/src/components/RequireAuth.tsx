import { useAuth } from "../contexts/auth";
import { useLocation, Navigate } from "react-router";

const RequireAuth = ({ children }: { children: JSX.Element }) => {
  let auth = useAuth();
  let location = useLocation();

  if (!auth.status.created) {
    return <Navigate to="/setup" state={{ from: location }} />;
  }

  if (!auth.status.running || !auth.status.authenticated) {
    return <Navigate to="/login" state={{ from: location }} />;
  }

  return children;
};

export default RequireAuth;
