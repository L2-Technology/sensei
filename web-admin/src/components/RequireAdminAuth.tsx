import { useAuth } from "../contexts/auth";
import { useLocation, Navigate } from "react-router";

const RequireAdminAuth = ({ children }: { children: JSX.Element }) => {
  let auth = useAuth();
  let location = useLocation();

  if (!auth.status.setup) {
    return <Navigate to="/setup" state={{ from: location }} />;
  }

  if (!auth.status.authenticatedAdmin) {
    return <Navigate to="/admin/login" state={{ from: location }} />;
  }

  return children;
};

export default RequireAdminAuth;
