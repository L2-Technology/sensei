import React from "react";
import "./App.css";
import { AuthProvider } from "./contexts/auth";
import AppLayout from "./layouts/AppLayout";
import AdminLoginPage from "./auth/pages/AdminLoginPage";
import NodeLoginPage from "./auth/pages/NodeLoginPage";
import SetupPage from "./auth/pages/SetupPage";
import RequireNodeAuth from "./components/RequireNodeAuth";
import RequireAdminAuth from "./components/RequireAdminAuth";
import { Routes, Route, Navigate } from "react-router-dom";
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
import AddPeerPage from "./peers/pages/AddPeerPage";
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
        <Route path="/login" element={<NodeLoginPage />} />
        <Route path="/logout" element={<LogoutPage />}/>
        <Route path="/setup" element={<SetupPage />} />
        <Route path="/admin/login" element={<AdminLoginPage />} />
        <Route path="/admin/logout" element={<LogoutPage />} />
        <Route
          path="/admin"
          element={
            <RequireAdminAuth>
              <AppLayout />
            </RequireAdminAuth>
          }
        >
          <Route path="/admin/nodes" element={<NodesPage />} />
          <Route path="/admin/nodes/new" element={<NewNodePage />} />
          <Route path="/admin/tokens" element={<TokensPage />} />
          <Route path="/admin/tokens/new" element={<NewTokenPage />} />
          <Route index element={<Navigate to="/admin/nodes" replace />} />
        </Route>
        <Route
          path="/"
          element={
            <RequireNodeAuth>
              <AppLayout />
            </RequireNodeAuth>
          }
          >
            <Route path="/chain" element={<ChainPage />} />
            <Route path="/fund" element={<FundPage />} />
            <Route path="/channels" element={<ChannelsPage />} />
            <Route path="/channels/open" element={<OpenChannelPage />} />
            <Route path="/payments" element={<PaymentsPage />} />
            <Route path="/receive-money" element={<ReceiveMoneyPage />} />
            <Route path="/send-money" element={<SendMoneyPage />} />
            <Route path="/peers" element={<PeersPage />} />
            <Route path="/peers/new" element={<AddPeerPage />} />
            <Route index element={<Navigate to="/chain" replace />} />
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
