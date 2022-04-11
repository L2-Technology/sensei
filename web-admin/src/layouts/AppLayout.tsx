import { Outlet } from "react-router";
import AdminNav from "./AdminNav";

const AppLayout = () => {

  return (
    <>
      <AdminNav />

      <div className="md:pl-64 flex flex-col flex-1 pt-10">
        <main className="flex-1 bg-gray-background min-h-screen text-white">
          <Outlet />
        </main>
      </div>
    </>
  );
};

export default AppLayout;
