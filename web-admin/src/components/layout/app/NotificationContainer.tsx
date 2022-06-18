import { Transition } from "@headlessui/react";
import { useNotification } from "../../../contexts/notification";
import { Fragment } from "react";
import { XIcon } from "@heroicons/react/outline";

const NotificationContainer = () => {
  const { hideNotification, component, isOpen, iconComponent } = useNotification();
  return (
    <>
      {/* Global notification live region, render this permanently at the end of the document */}
      <div
        aria-live="assertive"
        className="fixed inset-0 z-[5] flex items-end px-4 py-6 pointer-events-none sm:p-6 sm:items-start"
      >
        <div className="w-full flex flex-col items-center space-y-4 sm:items-end">
          <Transition
            show={isOpen}
            as={Fragment}
            enter="transform ease-out duration-300 transition"
            enterFrom="translate-y-2 opacity-0 sm:translate-y-0 sm:translate-x-2"
            enterTo="translate-y-0 opacity-100 sm:translate-x-0"
            leave="transition ease-in duration-100"
            leaveFrom="opacity-100"
            leaveTo="opacity-0"
          >
            <div className="max-w-sm w-full bg-plum  shadow-lg rounded-lg pointer-events-auto ring-1 ring-black ring-opacity-5 overflow-hidden">
              <div className="p-4">
                <div className="flex items-start">
                  <div className="flex-shrink-0">
                    {iconComponent}
                  </div>
                  <div className="ml-3 w-0 flex-1 pt-0.5">{component}</div>
                  <div className="ml-4 flex-shrink-0 flex">
                    <button
                      className="p-1 rounded-xl inline-flex text-light-plum focus:!outline-none hover:text-white focus:outline-none focus:ring-2 focus:ring-orange"
                      onClick={hideNotification}
                    >
                      <span className="sr-only">Close</span>
                      <XIcon className="h-5 w-5" aria-hidden="true" />
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </Transition>
        </div>
      </div>
    </>
  );
};

export default NotificationContainer;
