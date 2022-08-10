import sensei from "../../utils/sensei";
import { useEffect } from "react";
import { useNavigate } from "react-router";
import { useQueryClient } from "react-query";

const LogoutPage = () => {
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  useEffect(() => {
    const logout = async () => {
      await sensei.logout();
      queryClient.clear();
      navigate("/login", { replace: true });
    };

    logout();
  }, [navigate, queryClient]);

  return <div>...</div>;
};

export default LogoutPage;
