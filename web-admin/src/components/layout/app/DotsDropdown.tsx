import ClickableDiv from "src/components/layout/app/ClickableDiv";
import Transition from "src/components/Transition";
import { Link } from "react-router-dom";
import { useState } from "react";

interface DropdownOption {
  text: string;
  onClick?: () => void;
  href?: string;
}

interface DotsDropdownProps {
  options: DropdownOption[];
}

const DotsDropdown = ({ options }) => {
  const [isOpen, setIsOpen] = useState(false);
  return (
    <div className="relative inline-block text-left">
      <div>
        <button
          onClick={() => {
            setIsOpen(!isOpen);
          }}
          onBlur={() => {
            setTimeout(() => {
              setIsOpen(false);
            }, 100);
          }}
          className="flex items-center text-gray-400 hover:text-gray-600 focus:outline-none focus:text-gray-600"
          aria-label="Options"
          id="options-menu"
          aria-haspopup="true"
          aria-expanded="true"
        >
          <svg className="h-5 w-5" fill="currentColor" viewBox="0 0 20 20">
            <path d="M10 6a2 2 0 110-4 2 2 0 010 4zM10 12a2 2 0 110-4 2 2 0 010 4zM10 18a2 2 0 110-4 2 2 0 010 4z" />
          </svg>
        </button>
      </div>

      <Transition
        show={isOpen}
        enter="transition ease-out duration-100"
        enterFrom="transform opacity-0 scale-95"
        enterTo="transform opacity-100 scale-100"
        leave="transition ease-in duration-75"
        leaveFrom="transform opacity-100 scale-100"
        leaveTo="transform opacity-0 scale-95"
      >
        <div className="absolute right-0 mt-2 w-56 z-10 rounded-md shadow-lg">
          <div className="rounded-md bg-white ring-1 ring-black ring-opacity-5">
            <div
              className="py-1"
              role="menu"
              aria-orientation="vertical"
              aria-labelledby="options-menu"
            >
              {options.map((option) => {
                return (
                  <>
                    {option.href && (
                      <Link to={option.href}>
                        <a
                          className="block px-4 py-2 text-sm leading-5 text-gray-700 hover:bg-gray-100 hover:text-gray-900 focus:outline-none focus:bg-gray-100 focus:text-gray-900"
                          role="menuitem"
                        >
                          {option.text}
                        </a>
                      </Link>
                    )}

                    {option.onClick && (
                      <ClickableDiv
                        className="cursor-pointer block px-4 py-2 text-sm leading-5 text-gray-700 hover:bg-gray-100 hover:text-gray-900 focus:outline-none focus:bg-gray-100 focus:text-gray-900"
                        clickHandler={option.onClick}
                      >
                        {option.text}
                      </ClickableDiv>
                    )}
                  </>
                );
              })}
            </div>
          </div>
        </div>
      </Transition>
    </div>
  );
};

export default DotsDropdown;
