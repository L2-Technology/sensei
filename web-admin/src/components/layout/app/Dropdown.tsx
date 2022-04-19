import { Menu } from "@headlessui/react";
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
        <Menu.Items
          className={`${bodyClassName} absolute right-5 z-10 border border-gray-700 min-w-[160px] overflow-hidden rounded-xl bg-gray-background shadow-xl md:p-3`}
        >
          {items.map((item, i) => (
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
      </Menu>
    </div>
  );
}
