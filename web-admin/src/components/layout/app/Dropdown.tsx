import { Menu, Transition } from "@headlessui/react";
import { Fragment, PropsWithChildren, MouseEventHandler } from "react";
import { Link } from "react-router-dom";

interface ItemType {
  icon?: any;
  path?: string;
  label: string;
  className?: string;
  name?: string;
  onClick?: MouseEventHandler<HTMLButtonElement> | undefined;
}

interface DropdownProps {
  items?: ItemType[];
  bodyClassName?: string;
  btnClassName?: string;
  button: any;
}

export default function Dropdown({
  children,
  items = [],
  bodyClassName,
  btnClassName,
  button,
}: PropsWithChildren<DropdownProps>) {
  return (
    <div className="">
      <Menu>
        <Menu.Button
          className={`${btnClassName} rounded-xl p-2 hover:bg-white hover:bg-opacity-20`}
        >
          {button}
        </Menu.Button>
        <Transition
          as={Fragment}
          enter="transition ease-out duration-200"
          enterFrom="transform opacity-0 scale-95 translate-x-10"
          enterTo="transform opacity-100 scale-100 translate-x-0"
          leave="transition ease-in duration-100"
          leaveFrom="transform opacity-100 scale-100 translate-x-0"
          leaveTo="transform opacity-0 scale-95 translate-x-10"
        >
          <Menu.Items
            className={`${bodyClassName} absolute right-5 z-10 border border-gray-700 min-w-[160px] overflow-hidden rounded-xl bg-gray-background shadow-xl md:p-3`}
          >
            {items.map((item, _i) => (
              <Fragment key={item.label}>
                {item.path ? (
                  <Link
                    to={item.path}
                    className={`${item.className} flex w-full items-center space-x-2 p-3 text-sm hover:bg-white hover:bg-opacity-5 md:rounded-xl`}
                  >
                    {item.icon}
                    <span className="capitalize">{item.label}</span>
                  </Link>
                ) : (
                  <button
                    onClick={item.onClick}
                    className={`${item.className} flex w-full items-center space-x-2 p-2 text-sm hover:bg-white hover:bg-opacity-5 md:rounded-xl md:p-3`}
                  >
                    {item.icon}
                    <span className="capitalize">{item.label}</span>
                  </button>
                )}
              </Fragment>
            ))}
            {children}
          </Menu.Items>
        </Transition>
      </Menu>
    </div>
  );
}
