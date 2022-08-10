import { Fragment, useState } from "react";
import { Dialog, Transition, Menu } from "@headlessui/react";
import {
  AdjustmentsIcon,
  MenuIcon,
  CogIcon,
  LogoutIcon,
  XIcon,
  CashIcon,
  CollectionIcon,
  ShoppingCartIcon,
  QrcodeIcon,
  LinkIcon,
  KeyIcon,
  UserIcon,
  ChevronDownIcon,
  UsersIcon
} from "@heroicons/react/outline";
import { UserCircleIcon } from "@heroicons/react/solid";
import { useAuth } from "../contexts/auth";
import { NavLink, Link, useLocation } from "react-router-dom";
import SenseiLogo from "../components/icons/Sensei";

export default function AdminNav() {
  return (
    <>
      <div className="bottom-0 top-0 z-[2] hidden md:fixed md:flex md:w-52 md:flex-col lg:w-64">
        <AdminSidebar />
      </div>

      <header className="fixed top-0 left-0 right-0 z-[1] h-16 w-full bg-gray-senseihero shadow-md">
        <div className="flex h-full w-full items-center justify-between px-4 lg:px-6">
          <div className="flex items-center">
            <SidebarDrawer />

            <div className="flex flex-shrink-0 items-center md:opacity-0 px-2">
              <SenseiLogo className="h-8 w-auto" />
            </div>
          </div>

          <div className="flex items-center space-x-4">
            <UserDropdownMenu />
          </div>
        </div>
      </header>
    </>
  );
}

interface SidebarProps {
  setSidebarOpen?: React.Dispatch<React.SetStateAction<boolean>>;
}

const adminNav = [
  { name: "Nodes", href: "/admin/nodes", icon: CollectionIcon },
  { name: "Access Tokens", href: "/admin/tokens", icon: KeyIcon },
  { name: "Logout", href: "/admin/logout", icon: LogoutIcon },
];

const nodeNav = [
  { name: "Chain", href: "/chain", icon: LinkIcon },
  { name: "Channels", href: "/channels", icon: AdjustmentsIcon },
  { name: "Send Money", href: "/send-money", icon: ShoppingCartIcon },
  { name: "Receive Money", href: "/receive-money", icon: CashIcon },
  { name: "Peer Directory", href: "/peers", icon: UsersIcon },
  { name: "Logout", href: "/logout", icon: LogoutIcon },
];

export const AdminSidebar = ({ setSidebarOpen }: SidebarProps) => {
  const auth = useAuth();

  const navigation = auth.status.authenticatedAdmin ? adminNav : nodeNav;

  return (
    <div className="flex min-h-0 flex-1 flex-col bg-gray-senseihero">
      <div className="flex flex-shrink-0 items-center px-4 pt-5">
        <SenseiLogo className="h-10  w-auto" />
      </div>

      <div className="flex flex-1 flex-col overflow-y-auto  pt-5 pb-4">
        <nav className="mt-2 flex flex-col flex-1 space-y-1 px-2">
          {navigation.map((item) => (
            <NavLink
              onClick={() => setSidebarOpen && setSidebarOpen(false)}
              key={item.name}
              to={item.href}
              className={({ isActive }) => {
                return `${
                  isActive
                    ? "bg-orange text-white hover:bg-orange-hover"
                    : "text-gray-300 hover:bg-white hover:bg-opacity-5 hover:text-white"
                } group flex items-center rounded-xl px-3 py-2  text-sm font-medium`;
              }}
            >
              {({ isActive }) => {
                return (
                  <>
                    <item.icon
                      className={`${
                        isActive
                          ? "text-white"
                          : "text-gray-400 group-hover:text-gray-300"
                      } mr-3 h-6 w-6 flex-shrink-0`}
                      aria-hidden="true"
                    />
                    {item.name}
                  </>
                );
              }}
            </NavLink>
          ))}
        </nav>
        <div className="text-gray-400 pt-3 text-center">Sensei v{auth.status.version}</div>
      </div>
    </div>
  );
};

// Drawer sidebar for small screens
export const SidebarDrawer = () => {
  const [sidebarOpen, setSidebarOpen] = useState(false);

  return (
    <>
      <button
        type="button"
        className="relative flex h-10 w-10 items-center justify-center rounded-xl text-center text-white hover:bg-gray-accent1 md:hidden"
        onClick={() => setSidebarOpen(true)}
      >
        <span className="sr-only">Open sidebar</span>
        <MenuIcon className="h-6 w-6" aria-hidden="true" />
      </button>

      <Transition.Root show={sidebarOpen} as={Fragment}>
        <Dialog
          as="div"
          className="fixed inset-0 z-40 flex md:hidden"
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
            <Dialog.Overlay className="fixed inset-0 bg-black bg-opacity-50" />
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
            <div className="relative flex w-full max-w-xs flex-1 flex-col bg-gray-senseihero">
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
                    className="ml-1 flex h-10 w-10 items-center justify-center rounded-full focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
                    onClick={() => setSidebarOpen(false)}
                  >
                    <span className="sr-only">Close sidebar</span>
                    <XIcon className="h-6 w-6 text-white" aria-hidden="true" />
                  </button>
                </div>
              </Transition.Child>

              <AdminSidebar setSidebarOpen={setSidebarOpen} />
            </div>
          </Transition.Child>
          <div className="w-14 flex-shrink-0">
            {/* Force sidebar to shrink to fit close icon */}
          </div>
        </Dialog>
      </Transition.Root>
    </>
  );
};

export const UserDropdownMenu = () => {
  const auth = useAuth();

  const actionItems = [
    { name: "Logout", href: "/admin/logout", icon: LogoutIcon },
  ];

  return (
    <div className="relative hidden">
      <Menu>
        <Menu.Button className="h-9 w-9 rounded-full">
          <span className="sr-only">User</span>
          <UserCircleIcon className="h-9 w-9 text-white" />
        </Menu.Button>
        <Menu.Items className="absolute right-0 min-w-[200px] space-y-1 rounded-xl bg-gray-senseihero p-2 text-white shadow-2xl">
          <div className="my-3 flex items-center p-2 pt-0">
            <UserCircleIcon className="h-9 w-9 text-white" />

            <div className="ml-2 flex flex-col space-y-1">
              <p className="text-sm font-medium text-white">
                {auth.status.alias}
              </p>
              <p className="w-32 truncate text-xs font-medium text-gray-300">
                {auth.status.pubkey}
              </p>
            </div>
          </div>

          {actionItems.map((item, i) => (
            <Fragment key={item.name}>
              {actionItems.length - 1 === i && (
                <hr className="my-2 opacity-30" />
              )}
              <Menu.Item>
                <Link
                  to={item.href}
                  className="flex w-full items-center space-x-2 rounded-xl px-3 py-2 hover:bg-white hover:bg-opacity-5"
                >
                  <item.icon className="mr-2 h-6 w-6" />
                  <span className="capitalize">{item.name}</span>
                </Link>
              </Menu.Item>
            </Fragment>
          ))}
        </Menu.Items>
      </Menu>
    </div>
  );
};
