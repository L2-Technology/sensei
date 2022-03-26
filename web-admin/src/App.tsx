import React from "react";
import "./App.css";
import { AuthProvider } from "./contexts/auth";
import AppLayout from "./layouts/AppLayout";
import LoginPage from "./auth/pages/LoginPage";
import SetupPage from "./auth/pages/SetupPage";
import RequireAuth from "./components/RequireAuth";
import { Routes, Route } from "react-router-dom";
import { useQuery } from "react-query";
import getStatus from "./auth/queries/getStatus";
import NodesPage from "./nodes/pages/NodesPage";
import ChainPage from "./chain/pages/ChainPage";
import ChannelsPage from "./channels/pages/ChannelsPage";
import PaymentsPage from "./payments/pages/PaymentsPage";
import SendMoneyPage from "./payments/pages/SendMoneyPage";
import FundPage from "./chain/pages/FundPage";
import ReceiveMoneyPage from "./payments/pages/ReceiveMoneyPage";
import PeersPage from "./peers/pages/PeersPage";
import NewNodePage from "./nodes/pages/NewNodePage";
import Modal from "./components/layout/app/Modal";
import ConfirmModal from "./components/layout/app/ConfirmModal";
import ErrorModal from "./components/layout/app/ErrorModal";
import NotificationContainer from "./components/layout/app/NotificationContainer";
import OpenChannelPage from "./channels/pages/OpenChannelPage";
import LogoutPage from "./auth/pages/LogoutPage";
import TokensPage from "./tokens/pages/TokensPage";
import NewTokenPage from "./tokens/pages/NewTokenPage";

function App() {
  const { isLoading, data } = useQuery("status", getStatus, {
    refetchOnWindowFocus: false,
  });

  if (isLoading) {
    return <div></div>;
  }

  return (
    <AuthProvider initialStatus={data}>
      <Routes>
        <Route path="/admin/login" element={<LoginPage />} />
        <Route path="/admin/logout" element={<LogoutPage />} />
        <Route path="/admin/setup" element={<SetupPage />} />

        <Route
          path="/admin"
          element={
            <RequireAuth>
              <AppLayout />
            </RequireAuth>
          }
        >
          <Route path="/admin/chain" element={<ChainPage />} />
          <Route path="/admin/fund" element={<FundPage />} />
          <Route path="/admin/channels" element={<ChannelsPage />} />
          <Route path="/admin/channels/open" element={<OpenChannelPage />} />
          <Route path="/admin/payments" element={<PaymentsPage />} />
          <Route path="/admin/receive-money" element={<ReceiveMoneyPage />} />
          <Route path="/admin/send-money" element={<SendMoneyPage />} />
          <Route path="/admin/peers" element={<PeersPage />} />
          <Route path="/admin/nodes" element={<NodesPage />} />
          <Route path="/admin/nodes/new" element={<NewNodePage />} />
          <Route path="/admin/tokens" element={<TokensPage />} />
          <Route path="/admin/tokens/new" element={<NewTokenPage />} />
        </Route>
      </Routes>
      <Modal />
      <ErrorModal />
      <ConfirmModal />
      <NotificationContainer />
    </AuthProvider>
  );
}

export default App;
