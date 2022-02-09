import { Outlet } from "react-router";
import { Fragment, useState } from "react";
import { Dialog, Transition } from "@headlessui/react";
import {
  HomeIcon,
  AdjustmentsIcon,
  MenuIcon,
  CogIcon,
  LogoutIcon,
  XIcon,
  CashIcon,
  CollectionIcon,
  ShoppingCartIcon,
  QrcodeIcon,
  LinkIcon
} from "@heroicons/react/outline";

import { NavLink } from "react-router-dom";
import SenseiLogo from "../components/icons/Sensei";

import { useAuth } from "../contexts/auth";

const AppLayout = () => {
  const auth = useAuth();
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const navigation = []

  if (auth.isAdmin()) {
    navigation.push({
      name: "Nodes",
      href: "/admin/nodes",
      icon: CollectionIcon,
    });
  }

  navigation.push({
    name: "Fund Node",
    href: "/admin/fund",
    icon: QrcodeIcon,
  })

  navigation.push(
    {
      name: "Chain",
      href: "/admin/chain",
      icon: LinkIcon,
    },
  );

  navigation.push({
    name: "Channels",
    href: "/admin/channels",
    icon: AdjustmentsIcon,
  });

  navigation.push({
    name: "Send Money",
    href: "/admin/send-money",
    icon: ShoppingCartIcon,
  });

  navigation.push({
    name: "Receive Money",
    href: "/admin/receive-money",
    icon: CashIcon,
  });

  if (auth.isAdmin()) {
    navigation.push({
      name: "Config",
      href: "/admin/config",
      icon: CogIcon,
    });
  }
  navigation.push({ name: "Logout", href: "/admin/logout", icon: LogoutIcon });

  return (
    <>
      <div>
        <Transition.Root show={sidebarOpen} as={Fragment}>
          <Dialog
            as="div"
            className="fixed inset-0 flex z-40 md:hidden"
            onClose={setSidebarOpen}
          >
            <Transition.Child
              as={Fragment}
              enter="transition-opacity ease-linear duration-300"
              enterFrom="opacity-0"
              enterTo="opacity-100"
              leave="transition-opacity ease-linear duration-300"
              leaveFrom="opacity-100"
              leaveTo="opacity-0"
            >
              <Dialog.Overlay className="fixed inset-0 bg-gray-600 bg-opacity-75" />
            </Transition.Child>
            <Transition.Child
              as={Fragment}
              enter="transition ease-in-out duration-300 transform"
              enterFrom="-translate-x-full"
              enterTo="translate-x-0"
              leave="transition ease-in-out duration-300 transform"
              leaveFrom="translate-x-0"
              leaveTo="-translate-x-full"
            >
              <div className="relative flex-1 flex flex-col max-w-xs w-full bg-gray-senseihero">
                <Transition.Child
                  as={Fragment}
                  enter="ease-in-out duration-300"
                  enterFrom="opacity-0"
                  enterTo="opacity-100"
                  leave="ease-in-out duration-300"
                  leaveFrom="opacity-100"
                  leaveTo="opacity-0"
                >
                  <div className="absolute top-0 right-0 -mr-12 pt-2">
                    <button
                      type="button"
                      className="ml-1 flex items-center justify-center h-10 w-10 rounded-full focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
                      onClick={() => setSidebarOpen(false)}
                    >
                      <span className="sr-only">Close sidebar</span>
                      <XIcon
                        className="h-6 w-6 text-white"
                        aria-hidden="true"
                      />
                    </button>
                  </div>
                </Transition.Child>
                <div className="flex-1 h-0 pt-5 pb-4 overflow-y-auto">
                  <div className="flex-shrink-0 flex items-center">
                    <SenseiLogo className="h-14 w-auto" />
                  </div>
                  <nav className="mt-5 px-2 space-y-1">
                    {navigation.map((item) => (
                      <NavLink
                        key={item.name}
                        to={item.href}
                        className={({ isActive }) => {
                          return `${
                            isActive
                              ? "bg-gray-900 text-white"
                              : "text-gray-300 hover:bg-gray-700 hover:text-white"
                          } group flex items-center px-2 py-2 text-base font-medium rounded-md`;
                        }}
                      >
                        {({ isActive }) => {
                          return (
                            <>
                              <item.icon
                                className={`${
                                  isActive
                                    ? "text-gray-300"
                                    : "text-gray-400 group-hover:text-gray-300"
                                } mr-4 flex-shrink-0 h-6 w-6`}
                                aria-hidden="true"
                              />
                              {item.name}
                            </>
                          );
                        }}
                      </NavLink>
                    ))}
                  </nav>
                </div>
                <div className="flex-shrink-0 flex bg-gray-700 p-4">
                  <span className="flex-shrink-0 group block">
                    <div className="flex items-center">
                      <div>
                        <img
                          className="inline-block h-10 w-10 rounded-full"
                          src="https://images.unsplash.com/photo-1559087867-ce4c91325525?ixlib=rb-1.2.1&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=256&h=256&q=80"
                          alt=""
                        />
                      </div>
                      <div className="ml-3">
                        <p className="text-base font-medium text-white">
                          {auth.status.alias}
                        </p>
                        <p className="text-sm font-medium text-gray-400 truncate w-32">
                          {auth.status.pubkey}
                        </p>
                      </div>
                    </div>
                  </span>
                </div>
              </div>
            </Transition.Child>
            <div className="flex-shrink-0 w-14">
              {/* Force sidebar to shrink to fit close icon */}
            </div>
          </Dialog>
        </Transition.Root>

        {/* Static sidebar for desktop */}
        <div className="hidden md:flex md:w-64 md:flex-col md:fixed md:inset-y-0">
          {/* Sidebar component, swap this element with another sidebar if you like */}
          <div className="flex-1 flex flex-col min-h-0 bg-gray-senseihero">
            <div className="flex-1 flex flex-col pt-5 pb-4 overflow-y-auto">
              <div className="flex items-center flex-shrink-0 pb-4">
                <SenseiLogo className="h-10 w-auto pl-3"  />
              </div>
              <nav className="mt-2 flex-1 px-2 space-y-1">
                {navigation.map((item) => (
                  <NavLink
                    key={item.name}
                    to={item.href}
                    className={({ isActive }) => {
                      return `${
                        isActive
                          ? "bg-gray-900 text-white"
                          : "text-gray-300 hover:bg-gray-700 hover:text-white"
                      } group flex items-center px-2 py-2 text-sm font-medium rounded-md`;
                    }}
                  >
                    {({ isActive }) => {
                      return (
                        <>
                          <item.icon
                            className={`${
                              isActive
                                ? "text-gray-300"
                                : "text-gray-400 group-hover:text-gray-300"
                            } mr-3 flex-shrink-0 h-6 w-6`}
                            aria-hidden="true"
                          />
                          {item.name}
                        </>
                      );
                    }}
                  </NavLink>
                ))}
              </nav>
            </div>
            <div className="flex-shrink-0 flex bg-gray-accent1 p-4">
              <span className="flex-shrink-0 w-full group block">
                <div className="flex items-center">
                  <div>
                    <img
                      className="inline-block h-9 w-9 rounded-full"
                      src="https://images.unsplash.com/photo-1559087867-ce4c91325525?ixlib=rb-1.2.1&ixid=MnwxMjA3fDB8MHxwaG90by1wYWdlfHx8fGVufDB8fHx8&auto=format&fit=crop&w=256&h=256&q=80"
                      alt=""
                    />
                  </div>
                  <div className="ml-3">
                    <p className="text-sm font-medium text-white">
                      {auth.status.alias}
                    </p>
                    <p className="text-xs font-medium text-gray-300 truncate w-32">
                      {auth.status.pubkey}
                    </p>
                  </div>
                </div>
              </span>
            </div>
          </div>
        </div>
        <div className="md:pl-64 flex flex-col flex-1">
          <div className="sticky top-0 z-10 md:hidden pl-1 pt-1 sm:pl-3 sm:pt-3 bg-gray-100">
            <button
              type="button"
              className="-ml-0.5 -mt-0.5 h-12 w-12 inline-flex items-center justify-center rounded-md text-gray-500 hover:text-gray-900 focus:outline-none focus:ring-2 focus:ring-inset focus:ring-indigo-500"
              onClick={() => setSidebarOpen(true)}
            >
              <span className="sr-only">Open sidebar</span>
              <MenuIcon className="h-6 w-6" aria-hidden="true" />
            </button>
          </div>
          <main className="flex-1 bg-gray-background min-h-screen text-white">
            <Outlet />
          </main>
        </div>
      </div>
    </>
  );
};

export default AppLayout;
